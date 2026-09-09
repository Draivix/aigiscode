use std::collections::HashMap;
use tree_sitter::Node;

struct NamespaceScope {
    start: usize,
    end: usize,
    name: String,
    imports: HashMap<String, Option<String>>,
}

/// PHP namespace/import scopes, independent of filesystem layout.
pub(super) struct PhpNames {
    scopes: Vec<NamespaceScope>,
}

impl PhpNames {
    pub(super) fn new(root: Node<'_>, source: &str) -> Self {
        let namespaces = root
            .named_children(&mut root.walk())
            .filter(|node| node.kind() == "namespace_definition")
            .collect::<Vec<_>>();
        let mut scopes = vec![NamespaceScope {
            start: 0,
            end: source.len(),
            name: String::new(),
            imports: HashMap::new(),
        }];
        for (index, node) in namespaces.iter().enumerate() {
            scopes.push(NamespaceScope {
                start: node.start_byte(),
                end: if node.child_by_field_name("body").is_some() {
                    node.end_byte()
                } else {
                    namespaces
                        .get(index + 1)
                        .map_or(source.len(), Node::start_byte)
                },
                name: node
                    .child_by_field_name("name")
                    .map(|name| text(name, source).to_owned())
                    .unwrap_or_default(),
                imports: HashMap::new(),
            });
        }
        let mut names = Self { scopes };
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            if node.kind() == "namespace_use_declaration"
                && node.child_by_field_name("type").is_none()
            {
                let prefix = node
                    .named_children(&mut node.walk())
                    .find(|child| child.kind() == "namespace_name")
                    .map(|child| text(child, source))
                    .unwrap_or("");
                let body = node.child_by_field_name("body").unwrap_or(node);
                let scope = names.scope_index(node);
                for clause in body.named_children(&mut body.walk()).filter(|child| {
                    child.kind() == "namespace_use_clause"
                        && child.child_by_field_name("type").is_none()
                }) {
                    let alias = clause.child_by_field_name("alias");
                    let Some(target) = clause.named_children(&mut clause.walk()).find(|child| {
                        Some(*child) != alias && matches!(child.kind(), "name" | "qualified_name")
                    }) else {
                        continue;
                    };
                    let target = qualify(prefix, text(target, source).trim_start_matches('\\'));
                    let binding = alias
                        .map(|alias| text(alias, source))
                        .unwrap_or_else(|| target.rsplit('\\').next().unwrap_or(&target))
                        .to_ascii_lowercase();
                    names.scopes[scope]
                        .imports
                        .entry(binding)
                        .and_modify(|existing| {
                            if existing.as_ref() != Some(&target) {
                                *existing = None;
                            }
                        })
                        .or_insert(Some(target));
                }
            }
            stack.extend(node.named_children(&mut node.walk()));
        }
        names
    }

    fn scope_index(&self, node: Node<'_>) -> usize {
        self.scopes
            .iter()
            .rposition(|scope| scope.start <= node.start_byte() && node.end_byte() <= scope.end)
            .unwrap_or(0)
    }

    pub(super) fn declaration(&self, node: Node<'_>, name: &str) -> String {
        qualify(&self.scopes[self.scope_index(node)].name, name)
    }

    pub(super) fn resolve(&self, node: Node<'_>, name: &str) -> Option<String> {
        if name.starts_with('\\') {
            return Some(name.trim_start_matches('\\').to_owned());
        }
        if ["self", "static", "parent"]
            .iter()
            .any(|keyword| name.eq_ignore_ascii_case(keyword))
        {
            return None;
        }
        let scope = &self.scopes[self.scope_index(node)];
        let (head, tail) = name.split_once('\\').unwrap_or((name, ""));
        if head.eq_ignore_ascii_case("namespace") {
            return Some(qualify(&scope.name, tail));
        }
        if let Some(import) = scope.imports.get(&head.to_ascii_lowercase()) {
            return import.as_ref().map(|import| qualify(import, tail));
        }
        Some(qualify(&scope.name, name))
    }
}

fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or_default()
}

fn qualify(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else if name.is_empty() {
        prefix.to_owned()
    } else {
        format!("{prefix}\\{name}")
    }
}
