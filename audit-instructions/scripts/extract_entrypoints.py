#!/usr/bin/env python3
"""
extract_entrypoints.py – build the entry-point inventory from symbols.sqlite + route/registry files.

A missing entry point is the biggest risk of the whole audit (false "dead" code), so the inventory is
deliberately conservative and every entry carries the rule that produced it.

Two outputs:
  out/graph/entrypoints.json            – external entries (routes, commands, jobs, listeners, hooks,
                                          registries, callable arrays, schedule) → pass-2 batching roots
  out/graph/entrypoints.convention.json – convention roots (vendor-rooted public methods, entities, tools,
                                          namespace conventions) → reachability only, never batched
  out/graph/entrypoints.detail.json     – fqn → [rule, ...] for both sets
  out/graph/entrypoints.unresolved.json – class names referenced by registries that are not in the table

Usage (cwd = audited repo root):
  python3 extract_entrypoints.py --db lean-audit/out/symbols.sqlite --out lean-audit/out/graph
"""
import argparse, collections, json, pathlib, re, sqlite3

RESOURCE_METHODS = ["index", "create", "store", "show", "edit", "update", "destroy"]
EXTERNAL_BASES = {  # vendor base → which public methods are the externally triggered ones
    "Illuminate\\Console\\Command": ["handle", "__invoke"],
    "Illuminate\\Contracts\\Queue\\ShouldQueue": ["handle", "__invoke", "failed"],
    "Illuminate\\Support\\ServiceProvider": ["boot", "register"],
    "Illuminate\\Database\\Migrations\\Migration": ["up", "down"],
    "Illuminate\\Database\\Seeder": ["run"],
    "Illuminate\\Mail\\Mailable": ["build", "content", "envelope", "attachments"],
    "Illuminate\\Notifications\\Notification": ["via", "toMail", "toArray", "toDatabase", "toBroadcast"],
}
NAMESPACE_RULES = [  # (regex on class fqn, rule name) → all public methods become convention roots
    (r"\\Listeners\\", "listener_namespace"), (r"\\Subscribers\\", "subscriber_namespace"),
    (r"\\Entities\\", "orm_entity"), (r"\\Tools?\\", "tool_namespace"), (r"\\Hooks\\", "hook_class_namespace"),
    (r"\\Middleware\\", "middleware_namespace"), (r"\\Jobs\\", "job_namespace"), (r"\\Policies\\", "policy_namespace"),
    (r"\\Casts?\\", "cast_namespace"), (r"\\Rules\\", "rule_namespace"), (r"\\Observers\\", "observer_namespace"),
]
REGISTRY_GLOBS = ["config/*.php", "app/**/*.hook.php", "app/**/*.hooks.php", "app/**/*.actions.php", "app/**/*.fields.php",
                  "app/**/*.relations.php", "app/**/*.ui.php", "app/**/*.permissions.php", "app/**/*.notifications.php",
                  "app/**/*.activities.php", "app/**/module.php", "app/**/*.generated.php", "app/**/actions.php",
                  "app/**/config/*.php", "app/**/Config/*.php", "packages/**/config/*.php"]
ROUTE_GLOBS = ["routes/*.php", "app/**/routes/*.php", "packages/**/routes/*.php"]

USE_RE = re.compile(r"^use\s+([\w\\]+)(?:\s+as\s+(\w+))?\s*;", re.M)
CLASS_REF_RE = re.compile(r"([\w\\]+)::class")
CALLABLE_ARRAY_RE = re.compile(r"\[\s*([\w\\]+)::class\s*,\s*['\"](\w+)['\"]\s*\]")
APP_CALL_RE = re.compile(r"app\(\s*([\w\\]+)::class\s*\)\s*->\s*(\w+)\s*\(")
STATIC_CALL_RE = re.compile(r"(?<![\w$>])([A-Z][\w\\]+)::(\w+)\s*\(")
STRING_FQN_RE = re.compile(r"['\"]((?:App|Modules)\\\\[\w\\\\]+)['\"]")
AT_HANDLER_RE = re.compile(r"['\"]([\w\\]+)@(\w+)['\"]")
ROUTE_INVOKABLE_RE = re.compile(r"Route::(?:get|post|put|patch|delete|options|any|match)\s*\((?:\[[^\]]*\]|'[^']*'|\"[^\"]*\")\s*,\s*([\w\\]+)::class\s*\)")
ROUTE_RESOURCE_RE = re.compile(r"Route::(?:api)?[rR]esources?\s*\(\s*(?:'[^']*'|\"[^\"]*\")\s*,\s*([\w\\]+)::class")
ROUTE_CONTROLLER_RE = re.compile(r"(?:Route::controller|->controller)\s*\(\s*([\w\\]+)::class\s*\)")
ROUTE_STRING_METHOD_RE = re.compile(r"Route::(?:get|post|put|patch|delete|options|any|match)\s*\((?:\[[^\]]*\]|'[^']*'|\"[^\"]*\")\s*,\s*['\"](\w+)['\"]\s*\)")
ROUTE_CLOSURE_RE = re.compile(r"(?:Route|Artisan)::\w+\s*\((?:\[[^\]]*\]|'[^']*'|\"[^\"]*\")\s*,\s*(?:static\s+)?(?:function|fn)\b")


def load_symbols(db):
    classes, methods, functions = {}, collections.defaultdict(dict), []
    for kind, fqn, file, impl, vis, loc in db.execute("SELECT kind,fqn,file,implements,visibility,loc FROM symbol"):
        if kind in ("class", "interface", "trait", "enum"):
            classes[fqn] = {"file": file, "parents": json.loads(impl or "[]")}
        elif kind == "method":
            cls, m = fqn.rsplit("::", 1); methods[cls][m] = vis
        elif kind == "function":
            functions.append((fqn, file))
    return classes, methods, functions


def ancestors(cls, classes, seen=None):
    """All transitive parents/interfaces; names not in the table are returned too (vendor or unknown)."""
    seen = seen if seen is not None else set()
    for p in classes.get(cls, {}).get("parents", []):
        if p not in seen:
            seen.add(p); ancestors(p, classes, seen)
    return seen


class Inventory:
    def __init__(self, classes, methods):
        self.classes, self.methods = classes, methods
        self.external = collections.defaultdict(set)    # fqn → rules
        self.convention = collections.defaultdict(set)
        self.unresolved = collections.Counter()
        self.closures = []

    def add_method(self, cls, m, rule, external=True):
        if cls not in self.methods or m not in self.methods[cls]:
            if cls not in self.classes: self.unresolved[cls] += 1
            return False
        (self.external if external else self.convention)[f"{cls}::{m}"].add(rule); return True

    def add_public(self, cls, rule, external=True):
        if cls not in self.classes: self.unresolved[cls] += 1; return
        for m, vis in self.methods.get(cls, {}).items():
            if vis == "public": self.add_method(cls, m, rule, external)


def resolve(name, uses, ns):
    name = name.lstrip("\\")
    if name.startswith("App\\") or name.startswith("Illuminate\\"): return name
    head, _, rest = name.partition("\\")
    if head in uses: return uses[head] + ("\\" + rest if rest else "")
    return (ns + "\\" + name) if ns and "\\" not in name else name


def file_context(text):
    uses = {}
    for full, alias in USE_RE.findall(text): uses[alias or full.rsplit("\\", 1)[-1]] = full
    ns = re.search(r"^namespace\s+([\w\\]+)\s*;", text, re.M)
    return uses, (ns.group(1) if ns else "")


def scan_routes(repo, inv):
    for g in ROUTE_GLOBS:
        for p in sorted(repo.glob(g)):
            text = p.read_text(encoding="utf-8", errors="replace"); uses, ns = file_context(text); rel = str(p.relative_to(repo))
            for c, m in CALLABLE_ARRAY_RE.findall(text): inv.add_method(resolve(c, uses, ns), m, f"route:{rel}")
            for c in ROUTE_INVOKABLE_RE.findall(text):
                cls = resolve(c, uses, ns)
                if not inv.add_method(cls, "__invoke", f"route_invokable:{rel}"): inv.add_public(cls, f"route_invokable:{rel}")
            for c in ROUTE_RESOURCE_RE.findall(text):
                cls = resolve(c, uses, ns)
                for m in RESOURCE_METHODS: inv.add_method(cls, m, f"route_resource:{rel}")
            ctrls = [resolve(c, uses, ns) for c in ROUTE_CONTROLLER_RE.findall(text)]
            if ctrls:
                for m in ROUTE_STRING_METHOD_RE.findall(text):
                    for cls in ctrls: inv.add_method(cls, m, f"route_controller_group:{rel}")
            for c, m in AT_HANDLER_RE.findall(text): inv.add_method(resolve(c, uses, ns), m, f"route_at:{rel}")
            for mo in ROUTE_CLOSURE_RE.finditer(text):
                inv.closures.append({"file": rel, "line": text.count("\n", 0, mo.start()) + 1})
            if ROUTE_CLOSURE_RE.search(text):  # closure bodies: static + app() calls become roots
                for c, m in APP_CALL_RE.findall(text): inv.add_method(resolve(c, uses, ns), m, f"route_closure_callee:{rel}")
                for c, m in STATIC_CALL_RE.findall(text):
                    cls = resolve(c, uses, ns)
                    if cls in inv.classes: inv.add_method(cls, m, f"route_closure_callee:{rel}")


def scan_registries(repo, inv):
    seen = set()
    for g in REGISTRY_GLOBS:
        for p in sorted(repo.glob(g)):
            if p in seen or "/routes/" in str(p): continue
            seen.add(p); text = p.read_text(encoding="utf-8", errors="replace"); uses, ns = file_context(text); rel = str(p.relative_to(repo))
            for c, m in CALLABLE_ARRAY_RE.findall(text): inv.add_method(resolve(c, uses, ns), m, f"registry_callable:{rel}")
            for c, m in APP_CALL_RE.findall(text): inv.add_method(resolve(c, uses, ns), m, f"registry_app_call:{rel}")
            for c in CLASS_REF_RE.findall(text): inv.add_public(resolve(c, uses, ns), f"registry_class_ref:{rel}")
            for c in STRING_FQN_RE.findall(text): inv.add_public(c.replace("\\\\", "\\"), f"registry_string_fqn:{rel}")


def scan_callable_arrays(repo, inv):
    """Any [X::class, 'm'] literal anywhere in app/ is a dynamic dispatch target (events, pipelines, callbacks)."""
    for p in repo.glob("app/**/*.php"):
        if "/Tests/" in str(p) or "/tests/" in str(p): continue
        text = p.read_text(encoding="utf-8", errors="replace")
        if "::class," not in text and "::class ," not in text: continue
        uses, ns = file_context(text); rel = str(p.relative_to(repo))
        for c, m in CALLABLE_ARRAY_RE.findall(text): inv.add_method(resolve(c, uses, ns), m, f"callable_array:{rel}")


def scan_conventions(inv, functions):
    for cls in list(inv.classes):
        anc = ancestors(cls, inv.classes)
        vendor_rooted = any(a not in inv.classes for a in anc)
        for base, ms in EXTERNAL_BASES.items():
            if base in anc:
                for m in ms: inv.add_method(cls, m, f"framework_entry:{base.rsplit(chr(92), 1)[-1]}")
        if vendor_rooted: inv.add_public(cls, "vendor_rooted_public", external=False)
        for rx, rule in NAMESPACE_RULES:
            if re.search(rx, cls): inv.add_public(cls, rule, external=False)
    for fqn, file in functions:
        if file.startswith(("tools/", "database/")): inv.convention[fqn].add("script_function")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default="."); ap.add_argument("--db", required=True); ap.add_argument("--out", required=True)
    a = ap.parse_args()
    repo, out = pathlib.Path(a.repo).resolve(), pathlib.Path(a.out); out.mkdir(parents=True, exist_ok=True)
    classes, methods, functions = load_symbols(sqlite3.connect(a.db))
    inv = Inventory(classes, methods)
    scan_routes(repo, inv); scan_registries(repo, inv); scan_callable_arrays(repo, inv); scan_conventions(inv, functions)

    external = sorted(inv.external); convention = sorted(set(inv.convention) - set(inv.external))
    detail = {k: sorted(v) for k, v in list(inv.external.items()) + [(k, v) for k, v in inv.convention.items() if k not in inv.external]}
    (out / "entrypoints.json").write_text(json.dumps(external, indent=0))
    (out / "entrypoints.convention.json").write_text(json.dumps(convention, indent=0))
    (out / "entrypoints.detail.json").write_text(json.dumps(detail, indent=0, sort_keys=True))
    (out / "entrypoints.unresolved.json").write_text(json.dumps({"unresolved_classes": inv.unresolved.most_common(), "route_closures": inv.closures}, indent=1))
    rules = collections.Counter(r.split(":")[0] for v in detail.values() for r in v)
    print(f"external={len(external)} convention={len(convention)} unresolved_classes={len(inv.unresolved)} route_closures={len(inv.closures)}")
    for r, n in rules.most_common(): print(f"  {r:28s} {n}")


if __name__ == "__main__":
    main()
