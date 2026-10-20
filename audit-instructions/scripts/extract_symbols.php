#!/usr/bin/env php
<?php
/**
 * extract_symbols.php – projde repo přes nikic/php-parser a naplní out/symbols.sqlite
 *
 * Pro každou funkci/metodu/třídu uloží: FQN, soubor, řádky, LOC, signaturu, tělo
 * a NORMALIZOVANÝ AST HASH (proměnné → $v1,$v2…, literály → LIT, bez komentářů).
 * Shodný hash u dvou symbolů = klon typu 2 (přejmenované proměnné), nalezený bez LLM.
 *
 * Spuštění (z adresáře lean-audit/out/tools, kde je samostatný composer s nikic/php-parser):
 *   php ../../scripts/extract_symbols.php --root ../../../src --root ../../../app --db ../symbols.sqlite
 *
 * Nic nezapisuje do repa. Jen čte.
 */

declare(strict_types=1);

require __DIR__ . '/../out/tools/vendor/autoload.php';

use PhpParser\Node;
use PhpParser\NodeTraverser;
use PhpParser\NodeVisitorAbstract;
use PhpParser\NodeVisitor\NameResolver;
use PhpParser\ParserFactory;
use PhpParser\PrettyPrinter\Standard;

$opts = getopt('', ['root:', 'db:', 'exclude::']);
$roots = (array)($opts['root'] ?? ['src']);
$dbPath = $opts['db'] ?? __DIR__ . '/../out/symbols.sqlite';
$exclude = array_filter(explode(',', (string)($opts['exclude'] ?? 'vendor,node_modules,var,cache,storage,tests')));

$db = new PDO('sqlite:' . $dbPath);
$db->exec('PRAGMA journal_mode=WAL');
$db->exec(<<<SQL
CREATE TABLE IF NOT EXISTS symbol (
  id INTEGER PRIMARY KEY,
  kind TEXT NOT NULL,            -- class|interface|trait|enum|method|function|closure
  fqn TEXT NOT NULL,
  file TEXT NOT NULL,
  line_start INTEGER, line_end INTEGER, loc INTEGER,
  signature TEXT,
  ast_hash TEXT,                 -- normalizovaný
  body_tokens INTEGER,
  body TEXT,
  visibility TEXT, is_static INTEGER, is_abstract INTEGER,
  implements TEXT,               -- JSON seznam interface/parent u tříd; u metod: interface, které metodu deklaruje
  cluster_id TEXT, community_id TEXT,
  reachable_from_entry INTEGER, coverage_pct REAL, cpd_group TEXT,
  unused_public INTEGER, dead_code_detector INTEGER, churn_90d INTEGER
);
CREATE INDEX IF NOT EXISTS idx_hash ON symbol(ast_hash);
CREATE UNIQUE INDEX IF NOT EXISTS idx_fqn_file ON symbol(fqn, file, line_start);
SQL);

$parser = (new ParserFactory())->createForNewestSupportedVersion();
$printer = new Standard();

/** Normalizace: přejmenuje proměnné podle pořadí výskytu, literály nahradí LIT, zahodí komentáře. */
final class AstNormalizer extends NodeVisitorAbstract
{
    private array $vars = [];
    public function enterNode(Node $n)
    {
        $n->setAttribute('comments', []);
        if ($n instanceof Node\Expr\Variable && is_string($n->name) && $n->name !== 'this') {
            $this->vars[$n->name] ??= 'v' . (count($this->vars) + 1);
            $n->name = $this->vars[$n->name];
        }
        if ($n instanceof Node\Scalar\String_ || $n instanceof Node\Scalar\LNumber || $n instanceof Node\Scalar\DNumber) {
            return new Node\Scalar\String_('LIT');
        }
        return null;
    }
    public function reset(): void { $this->vars = []; }
}

final class Collector extends NodeVisitorAbstract
{
    public array $rows = [];
    private array $classStack = [];
    public function __construct(private string $file, private Standard $printer) {}

    public function enterNode(Node $n)
    {
        if ($n instanceof Node\Stmt\ClassLike) {
            $this->classStack[] = $n;
            $this->rows[] = $this->row($n, $this->kindOf($n), (string)($n->namespacedName ?? $n->name ?? 'anon'), null);
        }
        if ($n instanceof Node\Stmt\ClassMethod) {
            $cls = end($this->classStack);
            $fqn = ((string)($cls->namespacedName ?? $cls->name ?? 'anon')) . '::' . $n->name;
            $this->rows[] = $this->row($n, 'method', $fqn, $n);
        }
        if ($n instanceof Node\Stmt\Function_) {
            $this->rows[] = $this->row($n, 'function', (string)($n->namespacedName ?? $n->name), $n);
        }
        return null;
    }
    public function leaveNode(Node $n) { if ($n instanceof Node\Stmt\ClassLike) array_pop($this->classStack); return null; }

    private function kindOf(Node\Stmt\ClassLike $c): string
    {
        return match (true) { $c instanceof Node\Stmt\Interface_ => 'interface', $c instanceof Node\Stmt\Trait_ => 'trait', $c instanceof Node\Stmt\Enum_ => 'enum', default => 'class' };
    }

    private function row(Node $n, string $kind, string $fqn, ?Node $callable): array
    {
        $body = $callable?->stmts ? $this->printer->prettyPrint($callable->stmts) : '';
        $norm = '';
        if ($callable?->stmts) {
            $clone = unserialize(serialize($callable->stmts)); // deep copy
            $t = new NodeTraverser(); $nv = new AstNormalizer(); $t->addVisitor($nv);
            $norm = $this->printer->prettyPrint($t->traverse($clone));
            $norm = preg_replace('/\s+/', ' ', $norm);
        }
        $sig = $callable ? $this->printer->prettyPrint([$this->stripBody(clone $callable)]) : $kind . ' ' . $fqn;
        $impl = [];
        if ($n instanceof Node\Stmt\Class_) { if ($n->extends) $impl[] = (string)$n->extends; foreach ($n->implements as $i) $impl[] = (string)$i; }
        return [
            'kind' => $kind, 'fqn' => $fqn, 'file' => $this->file,
            'line_start' => $n->getStartLine(), 'line_end' => $n->getEndLine(),
            'loc' => $n->getEndLine() - $n->getStartLine() + 1,
            'signature' => trim(preg_replace('/\s+/', ' ', $sig)),
            'ast_hash' => $norm === '' ? null : hash('xxh3', $norm),
            'body_tokens' => $body === '' ? 0 : count(token_get_all('<?php ' . $body)),
            'body' => $body,
            'visibility' => $n instanceof Node\Stmt\ClassMethod ? ($n->isPublic() ? 'public' : ($n->isProtected() ? 'protected' : 'private')) : null,
            'is_static' => (int)($n instanceof Node\Stmt\ClassMethod && $n->isStatic()),
            'is_abstract' => (int)($n instanceof Node\Stmt\ClassMethod && $n->isAbstract()),
            'implements' => json_encode($impl),
        ];
    }
    private function stripBody(Node $n): Node { if (property_exists($n, 'stmts')) $n->stmts = []; return $n; }
}

$ins = $db->prepare('INSERT OR REPLACE INTO symbol (kind,fqn,file,line_start,line_end,loc,signature,ast_hash,body_tokens,body,visibility,is_static,is_abstract,implements)
 VALUES (:kind,:fqn,:file,:line_start,:line_end,:loc,:signature,:ast_hash,:body_tokens,:body,:visibility,:is_static,:is_abstract,:implements)');

$files = 0; $symbols = 0; $errors = 0;
foreach ($roots as $root) {
    $it = new RecursiveIteratorIterator(new RecursiveCallbackFilterIterator(new RecursiveDirectoryIterator($root, FilesystemIterator::SKIP_DOTS),
        fn($f) => !in_array($f->getFilename(), $exclude, true)));
    foreach ($it as $f) {
        if ($f->getExtension() !== 'php') continue;
        $files++;
        try {
            $ast = $parser->parse(file_get_contents($f->getPathname()));
            $t = new NodeTraverser(); $t->addVisitor(new NameResolver());
            $c = new Collector($f->getPathname(), $printer); $t->addVisitor($c);
            $t->traverse($ast);
            $db->beginTransaction();
            foreach ($c->rows as $r) { $ins->execute($r); $symbols++; }
            $db->commit();
        } catch (Throwable $e) {
            $errors++; fwrite(STDERR, "PARSE ERROR {$f->getPathname()}: {$e->getMessage()}\n");
        }
        if ($files % 500 === 0) fwrite(STDERR, "files=$files symbols=$symbols\n");
    }
}
fwrite(STDERR, "DONE files=$files symbols=$symbols errors=$errors\n");

// okamžité klony typu 2: stejný normalizovaný hash, ≥ 2 výskyty, netriviální tělo
$dups = $db->query('SELECT ast_hash, COUNT(*) c, GROUP_CONCAT(fqn, " | ") fqns FROM symbol WHERE ast_hash IS NOT NULL AND body_tokens >= 40 GROUP BY ast_hash HAVING c >= 2 ORDER BY c DESC')->fetchAll(PDO::FETCH_ASSOC);
file_put_contents(dirname($dbPath) . '/deterministic/clones_normalized.json', json_encode($dups, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES));
fwrite(STDERR, "normalized clone groups: " . count($dups) . "\n");
