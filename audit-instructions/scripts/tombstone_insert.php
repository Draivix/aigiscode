<?php

declare(strict_types=1);

/**
 * Deterministic tombstone insertion for lean-audit dead_symbol targets.
 *
 *   php tombstone_insert.php --root=<draivix worktree> --targets=<targets.jsonl> \
 *       [--filter=<regex on group>] [--write] [--report=<out.json>]
 *
 * For every target `Fqcn::method` (or `Fqcn` for a whole class) the file is resolved through the
 * repository's PSR-4 map, parsed with nikic/php-parser (taken from the worktree's vendor or the
 * shared checkout next to it), and `\App\Support\Tombstone::hit(__METHOD__); // lean-audit F-id`
 * is inserted as the first statement of the method body. Formatting of the rest of the file is
 * untouched (text splice, not a pretty-print). Methods without a body, bodies written on one line
 * and methods that already carry a tombstone are skipped and reported, so a model lane can take
 * them. Without --write nothing is changed; the report lists what would happen.
 */

use PhpParser\Node;
use PhpParser\Node\Stmt\ClassLike;
use PhpParser\Node\Stmt\ClassMethod;
use PhpParser\Node\Stmt\Function_;
use PhpParser\NodeFinder;
use PhpParser\NodeTraverser;
use PhpParser\NodeVisitor\NameResolver;
use PhpParser\ParserFactory;

$opts = getopt('', ['root:', 'targets:', 'filter::', 'write', 'report::']);
$root = rtrim((string) ($opts['root'] ?? ''), '/');
$targetsFile = (string) ($opts['targets'] ?? '');
$filter = isset($opts['filter']) ? (string) $opts['filter'] : null;
$write = array_key_exists('write', $opts);
$reportFile = isset($opts['report']) ? (string) $opts['report'] : null;

if ($root === '' || ! is_dir($root) || $targetsFile === '' || ! is_file($targetsFile)) {
    fwrite(STDERR, "usage: --root=<worktree> --targets=<targets.jsonl> [--filter=regex] [--write] [--report=file]\n");
    exit(2);
}

foreach ([$root.'/vendor/autoload.php', dirname($root).'/draivix/vendor/autoload.php'] as $autoload) {
    if (is_file($autoload)) {
        require $autoload;
        break;
    }
}
if (! class_exists(ParserFactory::class)) {
    fwrite(STDERR, "nikic/php-parser not found (need vendor/ in the worktree or in ../draivix)\n");
    exit(2);
}

$composer = json_decode((string) file_get_contents($root.'/composer.json'), true, 16, JSON_THROW_ON_ERROR);
/** @var array<string, string> $psr4 */
$psr4 = $composer['autoload']['psr-4'] ?? [];
uksort($psr4, static fn (string $a, string $b): int => strlen($b) <=> strlen($a));

$resolveFile = static function (string $class) use ($psr4, $root): ?string {
    foreach ($psr4 as $prefix => $dir) {
        if (str_starts_with($class, $prefix)) {
            $rel = rtrim($dir, '/').'/'.str_replace('\\', '/', substr($class, strlen($prefix))).'.php';

            return is_file($root.'/'.$rel) ? $rel : null;
        }
    }

    return null;
};

$groupOf = static function (string $rel): string {
    if (preg_match('#^app/Modules/([^/]+)/#', $rel, $m) === 1) {
        return 'module-'.$m[1];
    }
    if (preg_match('#^app/([^/]+)/#', $rel, $m) === 1) {
        return 'core-'.$m[1];
    }

    return 'other-'.explode('/', $rel)[0];
};

$parser = (new ParserFactory)->createForNewestSupportedVersion();
$finder = new NodeFinder;
$traverser = new NodeTraverser;
$traverser->addVisitor(new NameResolver);

/** @var array<string, list<array{pos: int, text: string, replaceLen: int}>> $edits */
$edits = [];
/** @var array<string, string> $sources */
$sources = [];
/** @var list<array<string, mixed>> $report */
$report = [];
/** @var array<string, true> $done */
$done = [];

$loadSource = static function (string $rel) use (&$sources, $root): string {
    return $sources[$rel] ??= (string) file_get_contents($root.'/'.$rel);
};

/** @param ClassMethod|Function_ $fn */
$planInsert = static function (Node\FunctionLike $fn, string $source, string $tag): array {
    $stmts = $fn->getStmts();
    if ($stmts === null) {
        return ['skip' => 'no_body'];
    }
    $ident = $fn instanceof ClassMethod ? '__METHOD__' : '__FUNCTION__';
    $stmt = '\\App\\Support\\Tombstone::hit('.$ident.'); // lean-audit '.$tag;
    $start = $fn->getStartFilePos();
    $end = $fn->getEndFilePos();
    $body = substr($source, $start, $end - $start + 1);
    if (str_contains($body, 'Tombstone::hit(')) {
        return ['skip' => 'already_tombstoned'];
    }
    $lineStart = strrpos(substr($source, 0, $start), "\n");
    $lineStart = $lineStart === false ? 0 : $lineStart + 1;
    $methodIndent = (string) preg_replace('/\S.*$/s', '', substr($source, $lineStart, $start - $lineStart));

    if ($stmts !== []) {
        $first = $stmts[0];
        // Insert above the first statement's own comments: an inline @var docblock
        // must stay directly on the statement it types.
        $comments = $first->getComments();
        $anchor = $comments !== [] ? $comments[0] : $first;
        $pos = $anchor->getStartFilePos();
        $prevNl = strrpos(substr($source, 0, $pos), "\n");
        if ($prevNl === false) {
            return ['skip' => 'inline_body'];
        }
        $prefix = substr($source, $prevNl + 1, $pos - $prevNl - 1);
        if (trim($prefix) !== '') {
            return ['skip' => 'inline_body'];
        }

        return ['pos' => $pos, 'text' => $stmt."\n".$prefix, 'replaceLen' => 0, 'line' => $anchor->getStartLine()];
    }

    // Empty body: `{}` or `{\n}` – rewrite the braces.
    $open = strrpos($body, '{');
    if ($open === false) {
        return ['skip' => 'no_brace'];
    }
    $openPos = $start + $open;
    $inner = substr($source, $openPos + 1, $end - $openPos - 1);
    if (trim($inner) !== '') {
        return ['skip' => 'unparsed_body'];
    }
    $text = "{\n".$methodIndent.'    '.$stmt."\n".$methodIndent.'}';

    return ['pos' => $openPos, 'text' => $text, 'replaceLen' => $end - $openPos + 1, 'line' => $fn->getStartLine()];
};

foreach (file($targetsFile, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES) ?: [] as $line) {
    $t = json_decode($line, true, 8, JSON_THROW_ON_ERROR);
    $symbol = (string) $t['symbol'];
    $tag = (string) $t['finding_id'];
    [$class, $method] = array_pad(explode('::', $symbol, 2), 2, null);
    $row = ['symbol' => $symbol, 'finding_id' => $tag, 'status' => $t['status'] ?? null];

    $rel = $resolveFile($class);
    if ($rel === null) {
        $report[] = $row + ['action' => 'skipped', 'reason' => 'file_not_found'];

        continue;
    }
    $group = $groupOf($rel);
    $row += ['file' => $rel, 'group' => $group];
    if ($filter !== null && preg_match('/'.$filter.'/', $group) !== 1) {
        $report[] = $row + ['action' => 'skipped', 'reason' => 'filtered'];

        continue;
    }
    if (isset($done[$symbol])) {
        $report[] = $row + ['action' => 'skipped', 'reason' => 'duplicate_target'];

        continue;
    }
    $done[$symbol] = true;

    $source = $loadSource($rel);
    try {
        $ast = $parser->parse($source);
    } catch (Throwable $e) {
        $report[] = $row + ['action' => 'skipped', 'reason' => 'parse_error: '.$e->getMessage()];

        continue;
    }
    $ast = $traverser->traverse($ast ?? []);

    $shortClass = substr((string) strrchr('\\'.$class, '\\'), 1);
    /** @var list<ClassLike> $classes */
    $classes = array_values(array_filter(
        $finder->findInstanceOf($ast, ClassLike::class),
        static fn (ClassLike $c): bool => $c->name !== null
            && ($c->namespacedName?->toString() === $class || $c->name->toString() === $shortClass),
    ));

    /** @var list<Node\FunctionLike> $targets */
    $targets = [];
    $reason = null;
    if ($method !== null) {
        foreach ($classes as $c) {
            foreach ($c->getMethods() as $m) {
                if ($m->name->toString() === $method) {
                    $targets[] = $m;
                }
            }
        }
        if ($targets === [] && $classes === []) {
            foreach ($finder->findInstanceOf($ast, Function_::class) as $f) {
                if ($f->name->toString() === $method) {
                    $targets[] = $f;
                }
            }
        }
        $reason = $targets === [] ? 'method_not_found' : null;
    } else {
        // Whole class: the constructor if it has a body, otherwise every public method with a body.
        foreach ($classes as $c) {
            $ctor = $c->getMethod('__construct');
            if ($ctor !== null && $ctor->getStmts() !== null) {
                $targets[] = $ctor;

                continue;
            }
            foreach ($c->getMethods() as $m) {
                if ($m->isPublic() && $m->getStmts() !== null) {
                    $targets[] = $m;
                }
            }
        }
        $reason = $classes === [] ? 'class_not_found' : ($targets === [] ? 'no_method_with_body' : null);
    }
    if ($reason !== null) {
        $report[] = $row + ['action' => 'skipped', 'reason' => $reason];

        continue;
    }

    foreach ($targets as $fn) {
        $plan = $planInsert($fn, $source, $tag);
        $name = $fn instanceof ClassMethod ? $fn->name->toString() : (string) $fn->name;
        if (isset($plan['skip'])) {
            $report[] = $row + ['action' => 'skipped', 'reason' => $plan['skip'], 'method' => $name, 'line' => $fn->getStartLine()];

            continue;
        }
        $edits[$rel][] = ['pos' => $plan['pos'], 'text' => $plan['text'], 'replaceLen' => $plan['replaceLen']];
        $report[] = $row + ['action' => $write ? 'inserted' : 'planned', 'method' => $name, 'line' => $plan['line']];
    }
}

$changed = 0;
if ($write) {
    foreach ($edits as $rel => $list) {
        usort($list, static fn (array $a, array $b): int => $b['pos'] <=> $a['pos']);
        $source = $sources[$rel];
        $lastPos = PHP_INT_MAX;
        foreach ($list as $e) {
            if ($e['pos'] + $e['replaceLen'] > $lastPos) {
                fwrite(STDERR, "overlapping edits in $rel, file skipped\n");

                continue 2;
            }
            $source = substr($source, 0, $e['pos']).$e['text'].substr($source, $e['pos'] + $e['replaceLen']);
            $lastPos = $e['pos'];
        }
        file_put_contents($root.'/'.$rel, $source);
        $changed++;
        exec('php -l '.escapeshellarg($root.'/'.$rel).' 2>&1', $out, $code);
        if ($code !== 0) {
            fwrite(STDERR, "SYNTAX ERROR after insertion in $rel: ".implode(' ', $out)."\n");
            file_put_contents($root.'/'.$rel, $sources[$rel]);
            $changed--;
        }
    }
}

$summary = [];
foreach ($report as $r) {
    $k = $r['action'].($r['action'] === 'skipped' ? ':'.$r['reason'] : '');
    $summary[$k] = ($summary[$k] ?? 0) + 1;
}
ksort($summary);
$byGroup = [];
foreach ($report as $r) {
    if ($r['action'] !== 'skipped' && isset($r['group'])) {
        $byGroup[$r['group']] = ($byGroup[$r['group']] ?? 0) + 1;
    }
}
arsort($byGroup);
fwrite(STDOUT, json_encode(['summary' => $summary, 'files_changed' => $changed, 'by_group' => $byGroup], JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES)."\n");
if ($reportFile !== null) {
    file_put_contents($reportFile, json_encode($report, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE)."\n");
}
