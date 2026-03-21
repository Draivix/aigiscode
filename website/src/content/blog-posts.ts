/* -------------------------------------------------------------------------- */
/*  Blog Post Data                                                            */
/* -------------------------------------------------------------------------- */

export interface BlogPost {
  slug: string;
  date: string;
  readTime: number;
  tags: string[];
  image?: string;
  title: Record<string, string>;
  description: Record<string, string>;
  metaDescription: Record<string, string>;
  content: Record<string, string>;
  author: { name: string; role: string };
  relatedSlugs: string[];
}

/* -------------------------------------------------------------------------- */
/*  Posts                                                                      */
/* -------------------------------------------------------------------------- */

export const blogPosts: BlogPost[] = [
  /* ======================================================================== */
  /*  0. Building a Semantic Code Graph in Rust                               */
  /* ======================================================================== */
  {
    slug: 'building-semantic-code-graph-rust',
    date: '2026-03-18',
    readTime: 12,
    tags: ['Engineering', 'Rust', 'Graph Architecture', 'Open Source'],
    image: '/blog-graphing-system.jpg',
    author: { name: 'David Strejc', role: 'Creator of AigisCode' },
    relatedSlugs: [
      'why-ai-code-analysis-matters-2026',
      'circular-dependencies-real-cost',
    ],
    title: {
      en: 'Building a Semantic Code Graph in Rust: How AigisCode Understands Your Codebase',
      cs: 'Budování sémantického grafu kódu v Rustu: Jak AigisCode rozumí vašemu codebase',
      fr: 'Construire un graphe de code sémantique en Rust : comment AigisCode comprend votre codebase',
      es: 'Construyendo un grafo de código semántico en Rust: cómo AigisCode entiende tu codebase',
      zh: '用 Rust 构建语义代码图：AigisCode 如何理解你的代码库',
      hi: 'Rust में सिमेंटिक कोड ग्राफ़ बनाना: AigisCode आपके कोडबेस को कैसे समझता है',
      pt: 'Construindo um Grafo de Código Semântico em Rust: Como o AigisCode Entende Seu Codebase',
      ar: 'بناء رسم بياني دلالي للكود في Rust: كيف يفهم AigisCode قاعدة الكود الخاصة بك',
      pl: 'Budowanie semantycznego grafu kodu w Rust: Jak AigisCode rozumie Twoją bazę kodu',
      bn: 'Rust-এ সিমান্টিক কোড গ্রাফ তৈরি: AigisCode কিভাবে আপনার কোডবেস বোঝে',
    },
    description: {
      en: 'A deep dive into how we built a typed, layered dependency graph in native Rust — and why flat code graphs fail for real architectural reasoning.',
      cs: 'Hluboký ponor do toho, jak jsme vybudovali typovaný, vrstvený graf závislostí v nativním Rustu — a proč ploché grafy kódu selhávají při skutečném architektonickém uvažování.',
      fr: 'Une plongée en profondeur dans la construction d\'un graphe de dépendances typé et stratifié en Rust natif — et pourquoi les graphes de code plats échouent pour le raisonnement architectural réel.',
      es: 'Una inmersión profunda en cómo construimos un grafo de dependencias tipado y estratificado en Rust nativo — y por qué los grafos de código planos fallan para el razonamiento arquitectónico real.',
      zh: '深入了解我们如何在原生 Rust 中构建类型化、分层的依赖图——以及为什么扁平代码图无法满足真正的架构推理。',
      hi: 'नेटिव Rust में हमने टाइप्ड, लेयर्ड डिपेंडेंसी ग्राफ़ कैसे बनाया — और फ्लैट कोड ग्राफ़ वास्तविक आर्किटेक्चरल रीज़निंग में क्यों विफल होते हैं, इसकी गहन जानकारी।',
      pt: 'Um mergulho profundo em como construímos um grafo de dependências tipado e em camadas em Rust nativo — e por que grafos de código planos falham para raciocínio arquitetural real.',
      ar: 'نظرة معمقة في كيفية بنائنا لرسم بياني للاعتماديات مصنّف ومتعدد الطبقات في Rust الأصلي — ولماذا تفشل الرسوم البيانية المسطحة في التحليل المعماري الحقيقي.',
      pl: 'Głęboki wgląd w to, jak zbudowaliśmy typowany, warstwowy graf zależności w natywnym Rust — i dlaczego płaskie grafy kodu zawodzą przy rzeczywistym wnioskowaniu architektonicznym.',
      bn: 'নেটিভ Rust-এ কিভাবে আমরা একটি টাইপড, লেয়ারড ডিপেন্ডেন্সি গ্রাফ তৈরি করলাম তার গভীর বিশ্লেষণ — এবং ফ্ল্যাট কোড গ্রাফ কেন প্রকৃত আর্কিটেকচারাল রিজনিংয়ে ব্যর্থ হয়।',
    },
    metaDescription: {
      en: 'Learn how AigisCode builds a semantic code graph in Rust with typed edges, layered meaning, and plugin-expanded framework behavior. Benchmarked against WordPress with 32,862 nodes and 95,878 relationships.',
      cs: 'Zjistěte, jak AigisCode buduje sémantický graf kódu v Rustu s typovanými hranami, vrstveným významem a chováním frameworků rozšířeným pluginy. Benchmarkováno na WordPress s 32 862 uzly a 95 878 relacemi.',
      fr: 'Découvrez comment AigisCode construit un graphe de code sémantique en Rust avec des arêtes typées, une signification stratifiée et un comportement de framework étendu par plugins. Benchmarké sur WordPress avec 32 862 nœuds et 95 878 relations.',
      es: 'Descubre cómo AigisCode construye un grafo de código semántico en Rust con aristas tipadas, significado estratificado y comportamiento de frameworks expandido por plugins. Probado con WordPress con 32.862 nodos y 95.878 relaciones.',
      zh: '了解 AigisCode 如何在 Rust 中构建语义代码图——具有类型化边、分层语义和插件扩展的框架行为。以 WordPress 为基准测试：32,862 个节点和 95,878 条关系。',
      hi: 'जानें कि AigisCode कैसे Rust में टाइप्ड एज, लेयर्ड मीनिंग और प्लगइन-विस्तारित फ़्रेमवर्क व्यवहार के साथ सिमेंटिक कोड ग्राफ़ बनाता है। WordPress के साथ बेंचमार्क: 32,862 नोड और 95,878 रिलेशनशिप।',
      pt: 'Saiba como o AigisCode constrói um grafo de código semântico em Rust com arestas tipadas, significado em camadas e comportamento de framework expandido por plugins. Benchmark contra WordPress com 32.862 nós e 95.878 relacionamentos.',
      ar: 'تعرّف كيف يبني AigisCode رسماً بيانياً دلالياً للكود في Rust مع حواف مصنّفة ومعنى متعدد الطبقات وسلوك إطار عمل موسّع بالإضافات. تم قياس الأداء مقابل WordPress بـ 32,862 عقدة و95,878 علاقة.',
      pl: 'Dowiedz się, jak AigisCode buduje semantyczny graf kodu w Rust z typowanymi krawędziami, warstwowym znaczeniem i zachowaniem frameworków rozszerzanym przez pluginy. Benchmarkowany na WordPress z 32 862 węzłami i 95 878 relacjami.',
      bn: 'জানুন কিভাবে AigisCode টাইপড এজ, লেয়ারড মিনিং এবং plugin-সম্প্রসারিত ফ্রেমওয়ার্ক আচরণ সহ Rust-এ একটি সিমান্টিক কোড গ্রাফ তৈরি করে। WordPress-এর বিপরীতে 32,862 নোড এবং 95,878 সম্পর্ক দিয়ে বেঞ্চমার্ক করা হয়েছে।',
    },
    content: {
      en: `
<p>Most static analysis tools treat your codebase as a bag of files. They scan each file in isolation, flag style violations, and move on. But real software architecture lives in the <strong>relationships between files</strong> — the imports, the calls, the inheritance chains, the event subscriptions, and the runtime dispatch patterns that wire everything together.</p>

<p>At AigisCode, we are building something different: a <strong>semantic code graph</strong> that captures not just what depends on what, but <em>how</em>, <em>why</em>, and <em>at what layer</em> those dependencies exist. This is the technical story of how we got here.</p>

<h2 id="why-flat-graphs-fail">Why Flat Code Graphs Fail</h2>

<p>A flat dependency graph says "file A depends on file B." That is useful, but limited. Consider a Laravel application where a controller calls a service, which dispatches a queued job, which resolves a repository through the IoC container. In a flat graph, you see four nodes and three edges. In reality, three different <em>kinds</em> of dependency are at play:</p>

<ul>
<li><strong>Structural</strong> — the <code>use</code> statement importing the service class</li>
<li><strong>Runtime</strong> — the queue dispatch that wires the job at runtime</li>
<li><strong>Framework</strong> — the container resolution that the IoC manages</li>
</ul>

<p>If you flatten all three into the same edge type, you lose the ability to reason about them differently. You cannot distinguish a structural cycle (always problematic) from a runtime cycle through the event bus (often intentional). You cannot tell whether a "dead" class is truly unreachable or simply resolved through a framework convention your tool does not understand.</p>

<p>This is the fundamental problem we set out to solve.</p>

<h2 id="the-canonical-rust-graph">The Canonical Rust Graph</h2>

<p>The source of truth in AigisCode is a semantic graph built entirely in native Rust. We chose Rust for the same reasons you would choose it for any performance-critical system: zero-cost abstractions, memory safety without a garbage collector, and the ability to process 30,000+ file codebases in under 25 seconds.</p>

<p>Every resolved edge in our graph carries typed metadata:</p>

<table>
<thead>
<tr><th>Field</th><th>Purpose</th></tr>
</thead>
<tbody>
<tr><td><code>ReferenceKind</code></td><td>What kind of reference (call, import, extends, implements)</td></tr>
<tr><td><code>RelationKind</code></td><td>The semantic relationship (dependency, inheritance, event)</td></tr>
<tr><td><code>GraphLayer</code></td><td>Structural, runtime, framework, or policy-overlay</td></tr>
<tr><td><code>EdgeStrength</code></td><td>How strong the coupling is</td></tr>
<tr><td><code>EdgeOrigin</code></td><td>Where the edge was discovered (parser, resolver, plugin)</td></tr>
<tr><td><code>ResolutionTier</code></td><td>How confident the resolution is</td></tr>
</tbody>
</table>

<p>This means every edge is not just "A depends on B" — it is "A depends on B <em>through this relation, at this layer, with this confidence, for this reason</em>." That distinction is critical for explainability and for the doctrine-based architectural judgment we are building toward.</p>

<h2 id="layered-meaning">Layered Meaning</h2>

<p>We moved away from a flat edge set early in development. The current model distinguishes four layers:</p>

<ol>
<li><strong>Structural edges</strong> — direct imports, class references, type annotations</li>
<li><strong>Runtime edges</strong> — queue dispatch, event emission, dynamic resolution</li>
<li><strong>Framework edges</strong> — IoC container bindings, WordPress hooks, Laravel service providers</li>
<li><strong>Policy-overlay edges</strong> — edges added by configuration rules for accepted codebase conventions</li>
</ol>

<p>This layering lets us ask fundamentally different questions against different graph views. We can detect structural cycles separately from runtime-expanded cycles. We can identify framework artifacts without confusing them with real coupling. And we can let users declare which patterns are intentional through policy rules, without modifying the core graph.</p>

<h2 id="plugin-expanded-framework-behavior">Plugin-Expanded Framework Behavior</h2>

<p>One of our most important architectural decisions is that framework knowledge does not live in the core language parsers. Instead, it lives in <strong>plugins</strong>:</p>

<ul>
<li>The <strong>queue plugin</strong> expands job dispatch into runtime edges</li>
<li>The <strong>container plugin</strong> resolves IoC bindings into framework edges</li>
<li>The <strong>WordPress plugin</strong> maps <code>add_action</code> / <code>do_action</code> into publish/subscribe edges</li>
</ul>

<p>The principle is simple:</p>

<ul>
<li>Language truth belongs in core</li>
<li>Framework truth belongs in plugins</li>
<li>Repository-specific accepted behavior belongs in policy rules</li>
</ul>

<p>Without this separation, the product would collapse into repository-specific hacks. Every WordPress installation would require different hardcoded patterns. Every Laravel version would break the graph. By keeping framework knowledge in plugins, we can evolve framework support independently of the core analysis engine.</p>

<h2 id="two-views-one-truth">Two Views, One Truth</h2>

<p>The most important correction in our latest iteration was separating the <strong>canonical graph</strong> from the <strong>dependency view</strong>.</p>

<p>Our initial graph exports were too noisy. They included synthetic MODULE nodes for every file, CONTAINS edges for every symbol, and repeated call-site edges counted individually. This made the graph look impressively large, but much of that size was representational overhead, not architectural value.</p>

<p>We now maintain two views from the same source of truth:</p>

<h3 id="canonical-graph">Canonical Graph (Evidence-Optimized)</h3>
<p>The canonical graph retains everything: repeated call sites, detailed runtime and plugin edges, fine-grained semantic information, and all evidence needed for deep investigation. This is what powers our detectors and AI review stage.</p>

<h3 id="dependency-view">Dependency View (Query-Optimized)</h3>
<p>The dependency view is a normalized projection that omits synthetic nodes, omits containment edges, remaps module-targeted edges onto file nodes, and collapses repeated dependencies into a single edge with an <code>occurrenceCount</code>. This is what powers our reporting, MCP access, and architecture exploration.</p>

<p>In other words: the canonical graph optimizes for truth and evidence. The dependency view optimizes for low-noise architectural interpretation.</p>

<h2 id="wordpress-benchmark">WordPress Benchmark: 32,862 Nodes in 22.78 Seconds</h2>

<p>We benchmark against WordPress — one of the largest and most complex PHP codebases in the world. Here are our current numbers from the normalized dependency view:</p>

<table>
<thead>
<tr><th>Metric</th><th>AigisCode</th></tr>
</thead>
<tbody>
<tr><td>Wall clock</td><td>22.78s</td></tr>
<tr><td>Nodes</td><td>32,862</td></tr>
<tr><td>Relationships</td><td>95,878</td></tr>
</tbody>
</table>

<p>The relationship breakdown reveals the richness of our graph:</p>

<ul>
<li><strong>CALL</strong>: 85,451 — function and method invocations</li>
<li><strong>EVENTPUBLISH</strong>: 3,662 — WordPress hook activations (<code>do_action</code>, <code>apply_filters</code>)</li>
<li><strong>OVERRIDES</strong>: 1,947 — method overrides in class hierarchies</li>
<li><strong>EVENTSUBSCRIBE</strong>: 1,868 — hook registrations (<code>add_action</code>, <code>add_filter</code>)</li>
<li><strong>EXTENDS</strong>: 1,489 — class inheritance</li>
<li><strong>IMPORT</strong>: 764 — file-level imports and includes</li>
<li><strong>TYPEUSE</strong>: 625 — type annotations and hints</li>
<li><strong>IMPLEMENTS</strong>: 72 — interface implementations</li>
</ul>

<p>The WordPress hook edges (EVENTPUBLISH + EVENTSUBSCRIBE) are particularly significant. These represent runtime wiring that flat static analysis tools completely miss. When WordPress calls <code>do_action('init')</code>, 47 different plugins respond. Our graph captures all 47 of those connections.</p>

<h2 id="optional-kuzu-read-model">Optional Kuzu Read Model</h2>

<p>For querying and exploration, we optionally export the dependency view into <a href="https://kuzudb.com/">Kuzu</a>, an embedded graph database. This gives us:</p>

<ul>
<li>Cypher query support for ad-hoc graph exploration</li>
<li>MCP server access for AI agents to query the graph</li>
<li>Fast pattern matching for architecture discovery</li>
</ul>

<p>The key architectural choice is that Kuzu is a <em>read model</em>, not the source of truth. Analysis logic should not be coupled to storage mechanics. The Rust graph and JSON artifacts remain the portable, canonical representation. Kuzu adds query power without creating storage dependency.</p>

<h2 id="what-this-enables">What This Enables</h2>

<p>With a typed, layered, evidence-preserving graph, we can build detectors and governance systems that were previously impossible:</p>

<ul>
<li><strong>Structural cycle detection</strong> that ignores intentional runtime cycles through event buses</li>
<li><strong>Dead code detection</strong> that understands framework-resolved classes are not truly dead</li>
<li><strong>God class identification</strong> that accounts for coupling at different graph layers</li>
<li><strong>Architecture surface generation</strong> that shows developers where the real pressure points are</li>
<li><strong>AI-powered review</strong> that classifies findings with full graph context, not just file-level heuristics</li>
</ul>

<p>This is the difference between a noisy code graph and a useful guardian system. AigisCode is not trying to count nodes and edges. It is trying to help humans and AI understand how a codebase is actually wired — where the architecture is healthy, where it is degrading, and what to do about it.</p>

<h2 id="try-it-yourself">Try It Yourself</h2>

<p>AigisCode is open source and MIT-licensed. You can run it on your own codebase today:</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
aigiscode analyze /path/to/your/project
</code></pre>

<p>The analysis produces structured JSON artifacts at <code>.aigiscode/</code> that any AI agent or CI pipeline can consume. We would love to hear what your graph looks like.</p>
`,
      cs: `
<p>Většina nástrojů pro statickou analýzu zachází s vaším codebase jako s pytlem souborů. Prohledají každý soubor izolovaně, označí porušení stylu a jdou dál. Ale skutečná softwarová architektura žije ve <strong>vztazích mezi soubory</strong> — v importech, voláních, řetězcích dědičnosti, odběrech událostí a vzorcích runtime dispatchingu, které vše propojují.</p>

<p>V AigisCode budujeme něco odlišného: <strong>sémantický graf kódu</strong>, který zachycuje nejen to, co závisí na čem, ale <em>jak</em>, <em>proč</em> a <em>na jaké vrstvě</em> tyto závislosti existují. Toto je technický příběh o tom, jak jsme se sem dostali.</p>

<h2 id="why-flat-graphs-fail">Proč ploché grafy kódu selhávají</h2>

<p>Plochý graf závislostí říká „soubor A závisí na souboru B." To je užitečné, ale omezené. Představte si Laravel aplikaci, kde kontroler volá službu, která odešle úlohu do fronty, která vyřeší repozitář přes IoC kontejner. V plochém grafu vidíte čtyři uzly a tři hrany. Ve skutečnosti jsou ve hře tři různé <em>druhy</em> závislostí:</p>

<ul>
<li><strong>Strukturální</strong> — příkaz <code>use</code> importující třídu služby</li>
<li><strong>Runtime</strong> — odeslání do fronty, které propojí úlohu za běhu</li>
<li><strong>Frameworkové</strong> — rozlišení kontejneru, které spravuje IoC</li>
</ul>

<p>Pokud všechny tři sloučíte do stejného typu hrany, ztratíte schopnost o nich uvažovat odlišně. Nemůžete rozlišit strukturální cyklus (vždy problematický) od runtime cyklu přes event bus (často záměrný). Nemůžete říct, zda je „mrtvá" třída skutečně nedosažitelná, nebo je jednoduše řešena přes konvenci frameworku, které váš nástroj nerozumí.</p>

<p>Toto je zásadní problém, který jsme se rozhodli vyřešit.</p>

<h2 id="the-canonical-rust-graph">Kanonický graf v Rustu</h2>

<p>Zdrojem pravdy v AigisCode je sémantický graf vybudovaný výhradně v nativním Rustu. Rust jsme zvolili ze stejných důvodů, proč byste ho zvolili pro jakýkoli výkonnostně kritický systém: bezeztrátové abstrakce, bezpečnost paměti bez garbage collectoru a schopnost zpracovat codebase s více než 30 000 soubory za méně než 25 sekund.</p>

<p>Každá vyřešená hrana v našem grafu nese typovaná metadata:</p>

<table>
<thead>
<tr><th>Pole</th><th>Účel</th></tr>
</thead>
<tbody>
<tr><td><code>ReferenceKind</code></td><td>Jaký druh reference (volání, import, extends, implements)</td></tr>
<tr><td><code>RelationKind</code></td><td>Sémantický vztah (závislost, dědičnost, událost)</td></tr>
<tr><td><code>GraphLayer</code></td><td>Strukturální, runtime, frameworková nebo policy-overlay</td></tr>
<tr><td><code>EdgeStrength</code></td><td>Jak silná je provázanost</td></tr>
<tr><td><code>EdgeOrigin</code></td><td>Kde byla hrana objevena (parser, resolver, plugin)</td></tr>
<tr><td><code>ResolutionTier</code></td><td>Jak spolehlivé je rozlišení</td></tr>
</tbody>
</table>

<p>To znamená, že každá hrana není jen „A závisí na B" — je to „A závisí na B <em>přes tento vztah, na této vrstvě, s touto spolehlivostí, z tohoto důvodu</em>." Toto rozlišení je klíčové pro vysvětlitelnost a pro doktrínou řízený architektonický úsudek, který budujeme.</p>

<h2 id="layered-meaning">Vrstvený význam</h2>

<p>Od ploché sady hran jsme se odklonili již v raných fázích vývoje. Současný model rozlišuje čtyři vrstvy:</p>

<ol>
<li><strong>Strukturální hrany</strong> — přímé importy, reference tříd, typové anotace</li>
<li><strong>Runtime hrany</strong> — odeslání do fronty, emise událostí, dynamické rozlišení</li>
<li><strong>Frameworkové hrany</strong> — vazby IoC kontejneru, WordPress hooky, Laravel service providery</li>
<li><strong>Policy-overlay hrany</strong> — hrany přidané konfiguračními pravidly pro přijaté konvence codebase</li>
</ol>

<p>Toto vrstvení nám umožňuje klást zásadně odlišné otázky proti různým pohledům na graf. Můžeme detekovat strukturální cykly odděleně od cyklů rozšířených o runtime. Můžeme identifikovat frameworkové artefakty, aniž bychom je zaměňovali se skutečnou provázaností. A uživatelům umožňujeme deklarovat, které vzory jsou záměrné, pomocí pravidel politik, aniž bychom modifikovali jádro grafu.</p>

<h2 id="plugin-expanded-framework-behavior">Chování frameworků rozšířené pluginy</h2>

<p>Jedním z našich nejdůležitějších architektonických rozhodnutí je, že znalost frameworku nežije v jádrových jazykových parserech. Místo toho žije v <strong>pluginech</strong>:</p>

<ul>
<li><strong>Queue plugin</strong> rozšiřuje odeslání úloh na runtime hrany</li>
<li><strong>Container plugin</strong> řeší IoC vazby na frameworkové hrany</li>
<li><strong>WordPress plugin</strong> mapuje <code>add_action</code> / <code>do_action</code> na publish/subscribe hrany</li>
</ul>

<p>Princip je jednoduchý:</p>

<ul>
<li>Jazyková pravda patří do jádra</li>
<li>Frameworková pravda patří do pluginů</li>
<li>Repozitářově specifické přijaté chování patří do pravidel politik</li>
</ul>

<p>Bez tohoto oddělení by se produkt rozpadl na repozitářově specifické hacky. Každá instalace WordPress by vyžadovala jiné hardcodované vzory. Každá verze Laravelu by rozbila graf. Uchováním znalostí frameworku v pluginech můžeme vyvíjet podporu frameworků nezávisle na jádrovém analytickém enginu.</p>

<h2 id="two-views-one-truth">Dva pohledy, jedna pravda</h2>

<p>Nejdůležitější korekce v naší poslední iteraci bylo oddělení <strong>kanonického grafu</strong> od <strong>pohledu závislostí</strong>.</p>

<p>Naše počáteční exporty grafu byly příliš zašuměné. Zahrnovaly syntetické MODULE uzly pro každý soubor, CONTAINS hrany pro každý symbol a opakované hrany míst volání počítané jednotlivě. Díky tomu graf vypadal impozantně velký, ale velká část této velikosti byla reprezentační režie, nikoli architektonická hodnota.</p>

<p>Nyní udržujeme dva pohledy ze stejného zdroje pravdy:</p>

<h3 id="canonical-graph">Kanonický graf (optimalizovaný pro evidenci)</h3>
<p>Kanonický graf uchovává vše: opakovaná místa volání, detailní runtime a pluginové hrany, jemnozrnné sémantické informace a veškerou evidenci potřebnou pro hloubkové zkoumání. Toto pohání naše detektory a fázi AI review.</p>

<h3 id="dependency-view">Pohled závislostí (optimalizovaný pro dotazy)</h3>
<p>Pohled závislostí je normalizovaná projekce, která vynechává syntetické uzly, vynechává hrany obsahování, přemapovává hrany cílené na moduly na souborové uzly a slučuje opakované závislosti do jedné hrany s <code>occurrenceCount</code>. Toto pohání naše reportování, MCP přístup a prozkoumávání architektury.</p>

<p>Jinými slovy: kanonický graf optimalizuje pro pravdu a evidenci. Pohled závislostí optimalizuje pro nízko-šumovou architektonickou interpretaci.</p>

<h2 id="wordpress-benchmark">WordPress benchmark: 32 862 uzlů za 22,78 sekundy</h2>

<p>Provádíme benchmark na WordPress — jednom z největších a nejsložitějších PHP codebase na světě. Zde jsou naše aktuální čísla z normalizovaného pohledu závislostí:</p>

<table>
<thead>
<tr><th>Metrika</th><th>AigisCode</th></tr>
</thead>
<tbody>
<tr><td>Celkový čas</td><td>22.78s</td></tr>
<tr><td>Uzly</td><td>32,862</td></tr>
<tr><td>Relace</td><td>95,878</td></tr>
</tbody>
</table>

<p>Rozklad relací odhaluje bohatost našeho grafu:</p>

<ul>
<li><strong>CALL</strong>: 85,451 — volání funkcí a metod</li>
<li><strong>EVENTPUBLISH</strong>: 3,662 — aktivace WordPress hooků (<code>do_action</code>, <code>apply_filters</code>)</li>
<li><strong>OVERRIDES</strong>: 1,947 — přepsání metod v hierarchiích tříd</li>
<li><strong>EVENTSUBSCRIBE</strong>: 1,868 — registrace hooků (<code>add_action</code>, <code>add_filter</code>)</li>
<li><strong>EXTENDS</strong>: 1,489 — dědičnost tříd</li>
<li><strong>IMPORT</strong>: 764 — importy a vkládání na úrovni souborů</li>
<li><strong>TYPEUSE</strong>: 625 — typové anotace a nápovědy</li>
<li><strong>IMPLEMENTS</strong>: 72 — implementace rozhraní</li>
</ul>

<p>Hrany WordPress hooků (EVENTPUBLISH + EVENTSUBSCRIBE) jsou obzvláště významné. Představují runtime propojení, které ploché nástroje statické analýzy zcela přehlížejí. Když WordPress zavolá <code>do_action('init')</code>, reaguje 47 různých pluginů. Náš graf zachycuje všech 47 těchto propojení.</p>

<h2 id="optional-kuzu-read-model">Volitelný Kuzu read model</h2>

<p>Pro dotazování a průzkum volitelně exportujeme pohled závislostí do <a href="https://kuzudb.com/">Kuzu</a>, vestavěné grafové databáze. To nám dává:</p>

<ul>
<li>Podporu Cypher dotazů pro ad-hoc průzkum grafu</li>
<li>MCP server přístup pro AI agenty k dotazování grafu</li>
<li>Rychlé porovnávání vzorů pro objevování architektury</li>
</ul>

<p>Klíčové architektonické rozhodnutí je, že Kuzu je <em>read model</em>, nikoli zdroj pravdy. Analytická logika by neměla být svázaná s mechanikou úložiště. Rust graf a JSON artefakty zůstávají přenositelnou, kanonickou reprezentací. Kuzu přidává sílu dotazování bez vytváření závislosti na úložišti.</p>

<h2 id="what-this-enables">Co to umožňuje</h2>

<p>S typovaným, vrstveným grafem uchovávajícím evidenci můžeme budovat detektory a řídicí systémy, které byly dříve nemožné:</p>

<ul>
<li><strong>Detekce strukturálních cyklů</strong>, která ignoruje záměrné runtime cykly přes event busy</li>
<li><strong>Detekce mrtvého kódu</strong>, která rozumí tomu, že třídy řešené frameworkem nejsou skutečně mrtvé</li>
<li><strong>Identifikace god tříd</strong>, která zohledňuje provázanost na různých vrstvách grafu</li>
<li><strong>Generování architektonického povrchu</strong>, které vývojářům ukazuje, kde jsou skutečné tlakové body</li>
<li><strong>AI review</strong>, které klasifikuje nálezy s plným kontextem grafu, nejen heuristikami na úrovni souborů</li>
</ul>

<p>Toto je rozdíl mezi zašuměným grafem kódu a užitečným strážním systémem. AigisCode se nesnaží počítat uzly a hrany. Snaží se pomoci lidem a AI porozumět tomu, jak je codebase skutečně propojen — kde je architektura zdravá, kde degraduje a co s tím dělat.</p>

<h2 id="try-it-yourself">Vyzkoušejte si to sami</h2>

<p>AigisCode je open source a licencovaný pod MIT. Můžete ho spustit na vlastním codebase ještě dnes:</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
aigiscode analyze /path/to/your/project
</code></pre>

<p>Analýza vytváří strukturované JSON artefakty v <code>.aigiscode/</code>, které může zpracovat jakýkoli AI agent nebo CI pipeline. Rádi uslyšíme, jak vypadá váš graf.</p>
`,
      fr: `
<p>La plupart des outils d'analyse statique traitent votre codebase comme un sac de fichiers. Ils analysent chaque fichier isolément, signalent les violations de style et passent au suivant. Mais la véritable architecture logicielle réside dans les <strong>relations entre les fichiers</strong> — les imports, les appels, les chaînes d'héritage, les abonnements aux événements et les modèles de dispatch runtime qui connectent le tout.</p>

<p>Chez AigisCode, nous construisons quelque chose de différent : un <strong>graphe de code sémantique</strong> qui capture non seulement ce qui dépend de quoi, mais <em>comment</em>, <em>pourquoi</em> et <em>à quelle couche</em> ces dépendances existent. Voici l'histoire technique de comment nous en sommes arrivés là.</p>

<h2 id="why-flat-graphs-fail">Pourquoi les graphes de code plats échouent</h2>

<p>Un graphe de dépendances plat dit « le fichier A dépend du fichier B. » C'est utile, mais limité. Considérez une application Laravel où un contrôleur appelle un service, qui dispatche un job en file d'attente, qui résout un repository via le conteneur IoC. Dans un graphe plat, vous voyez quatre nœuds et trois arêtes. En réalité, trois <em>types</em> différents de dépendance sont en jeu :</p>

<ul>
<li><strong>Structurelle</strong> — la déclaration <code>use</code> important la classe de service</li>
<li><strong>Runtime</strong> — le dispatch de file d'attente qui connecte le job à l'exécution</li>
<li><strong>Framework</strong> — la résolution du conteneur gérée par l'IoC</li>
</ul>

<p>Si vous aplatissez ces trois types dans le même type d'arête, vous perdez la capacité de raisonner différemment sur chacun. Vous ne pouvez pas distinguer un cycle structurel (toujours problématique) d'un cycle runtime via le bus d'événements (souvent intentionnel). Vous ne pouvez pas dire si une classe « morte » est véritablement inaccessible ou simplement résolue via une convention de framework que votre outil ne comprend pas.</p>

<p>C'est le problème fondamental que nous avons entrepris de résoudre.</p>

<h2 id="the-canonical-rust-graph">Le graphe canonique en Rust</h2>

<p>La source de vérité dans AigisCode est un graphe sémantique construit entièrement en Rust natif. Nous avons choisi Rust pour les mêmes raisons que vous le choisiriez pour tout système critique en performance : abstractions à coût zéro, sécurité mémoire sans ramasse-miettes, et la capacité de traiter des codebases de plus de 30 000 fichiers en moins de 25 secondes.</p>

<p>Chaque arête résolue dans notre graphe porte des métadonnées typées :</p>

<table>
<thead>
<tr><th>Champ</th><th>Objectif</th></tr>
</thead>
<tbody>
<tr><td><code>ReferenceKind</code></td><td>Quel type de référence (appel, import, extends, implements)</td></tr>
<tr><td><code>RelationKind</code></td><td>La relation sémantique (dépendance, héritage, événement)</td></tr>
<tr><td><code>GraphLayer</code></td><td>Structurelle, runtime, framework ou policy-overlay</td></tr>
<tr><td><code>EdgeStrength</code></td><td>La force du couplage</td></tr>
<tr><td><code>EdgeOrigin</code></td><td>Où l'arête a été découverte (parser, resolver, plugin)</td></tr>
<tr><td><code>ResolutionTier</code></td><td>Le niveau de confiance de la résolution</td></tr>
</tbody>
</table>

<p>Cela signifie que chaque arête n'est pas juste « A dépend de B » — c'est « A dépend de B <em>via cette relation, à cette couche, avec cette confiance, pour cette raison</em>. » Cette distinction est essentielle pour l'explicabilité et pour le jugement architectural basé sur la doctrine que nous construisons.</p>

<h2 id="layered-meaning">Signification stratifiée</h2>

<p>Nous avons abandonné l'ensemble d'arêtes plat tôt dans le développement. Le modèle actuel distingue quatre couches :</p>

<ol>
<li><strong>Arêtes structurelles</strong> — imports directs, références de classes, annotations de types</li>
<li><strong>Arêtes runtime</strong> — dispatch de files d'attente, émission d'événements, résolution dynamique</li>
<li><strong>Arêtes framework</strong> — liaisons du conteneur IoC, hooks WordPress, service providers Laravel</li>
<li><strong>Arêtes policy-overlay</strong> — arêtes ajoutées par des règles de configuration pour les conventions acceptées du codebase</li>
</ol>

<p>Cette stratification nous permet de poser des questions fondamentalement différentes sur différentes vues du graphe. Nous pouvons détecter les cycles structurels séparément des cycles étendus par le runtime. Nous pouvons identifier les artefacts de framework sans les confondre avec du couplage réel. Et nous pouvons laisser les utilisateurs déclarer quels patterns sont intentionnels via des règles de politique, sans modifier le graphe central.</p>

<h2 id="plugin-expanded-framework-behavior">Comportement de framework étendu par plugins</h2>

<p>L'une de nos décisions architecturales les plus importantes est que la connaissance du framework ne réside pas dans les parseurs de langage centraux. Elle réside plutôt dans les <strong>plugins</strong> :</p>

<ul>
<li>Le <strong>plugin queue</strong> étend le dispatch de jobs en arêtes runtime</li>
<li>Le <strong>plugin container</strong> résout les liaisons IoC en arêtes framework</li>
<li>Le <strong>plugin WordPress</strong> mappe <code>add_action</code> / <code>do_action</code> en arêtes publish/subscribe</li>
</ul>

<p>Le principe est simple :</p>

<ul>
<li>La vérité du langage appartient au cœur</li>
<li>La vérité du framework appartient aux plugins</li>
<li>Le comportement accepté spécifique au dépôt appartient aux règles de politique</li>
</ul>

<p>Sans cette séparation, le produit s'effondrerait en hacks spécifiques aux dépôts. Chaque installation WordPress nécessiterait des patterns codés en dur différents. Chaque version de Laravel casserait le graphe. En gardant la connaissance du framework dans les plugins, nous pouvons faire évoluer le support des frameworks indépendamment du moteur d'analyse central.</p>

<h2 id="two-views-one-truth">Deux vues, une vérité</h2>

<p>La correction la plus importante dans notre dernière itération a été la séparation du <strong>graphe canonique</strong> de la <strong>vue des dépendances</strong>.</p>

<p>Nos premiers exports de graphe étaient trop bruyants. Ils incluaient des nœuds MODULE synthétiques pour chaque fichier, des arêtes CONTAINS pour chaque symbole, et des arêtes de sites d'appel répétées comptées individuellement. Cela donnait l'impression d'un graphe impressionnant par sa taille, mais une grande partie de cette taille était de la surcharge représentationnelle, pas de la valeur architecturale.</p>

<p>Nous maintenons désormais deux vues à partir de la même source de vérité :</p>

<h3 id="canonical-graph">Graphe canonique (optimisé pour l'évidence)</h3>
<p>Le graphe canonique conserve tout : les sites d'appel répétés, les arêtes runtime et plugin détaillées, les informations sémantiques fines, et toute l'évidence nécessaire pour une investigation approfondie. C'est ce qui alimente nos détecteurs et l'étape de revue IA.</p>

<h3 id="dependency-view">Vue des dépendances (optimisée pour les requêtes)</h3>
<p>La vue des dépendances est une projection normalisée qui omet les nœuds synthétiques, omet les arêtes de contenance, remappe les arêtes ciblant des modules sur les nœuds fichier, et fusionne les dépendances répétées en une seule arête avec un <code>occurrenceCount</code>. C'est ce qui alimente notre reporting, l'accès MCP et l'exploration de l'architecture.</p>

<p>Autrement dit : le graphe canonique optimise pour la vérité et l'évidence. La vue des dépendances optimise pour une interprétation architecturale à faible bruit.</p>

<h2 id="wordpress-benchmark">Benchmark WordPress : 32 862 nœuds en 22,78 secondes</h2>

<p>Nous nous benchmarkons contre WordPress — l'un des codebases PHP les plus grands et les plus complexes au monde. Voici nos chiffres actuels depuis la vue des dépendances normalisée :</p>

<table>
<thead>
<tr><th>Métrique</th><th>AigisCode</th></tr>
</thead>
<tbody>
<tr><td>Temps total</td><td>22.78s</td></tr>
<tr><td>Nœuds</td><td>32,862</td></tr>
<tr><td>Relations</td><td>95,878</td></tr>
</tbody>
</table>

<p>La répartition des relations révèle la richesse de notre graphe :</p>

<ul>
<li><strong>CALL</strong> : 85,451 — invocations de fonctions et méthodes</li>
<li><strong>EVENTPUBLISH</strong> : 3,662 — activations de hooks WordPress (<code>do_action</code>, <code>apply_filters</code>)</li>
<li><strong>OVERRIDES</strong> : 1,947 — surcharges de méthodes dans les hiérarchies de classes</li>
<li><strong>EVENTSUBSCRIBE</strong> : 1,868 — enregistrements de hooks (<code>add_action</code>, <code>add_filter</code>)</li>
<li><strong>EXTENDS</strong> : 1,489 — héritage de classes</li>
<li><strong>IMPORT</strong> : 764 — imports et inclusions au niveau fichier</li>
<li><strong>TYPEUSE</strong> : 625 — annotations et indications de types</li>
<li><strong>IMPLEMENTS</strong> : 72 — implémentations d'interfaces</li>
</ul>

<p>Les arêtes de hooks WordPress (EVENTPUBLISH + EVENTSUBSCRIBE) sont particulièrement significatives. Elles représentent le câblage runtime que les outils d'analyse statique plats manquent complètement. Quand WordPress appelle <code>do_action('init')</code>, 47 plugins différents répondent. Notre graphe capture les 47 connexions.</p>

<h2 id="optional-kuzu-read-model">Read model Kuzu optionnel</h2>

<p>Pour le requêtage et l'exploration, nous exportons optionnellement la vue des dépendances vers <a href="https://kuzudb.com/">Kuzu</a>, une base de données graphe embarquée. Cela nous donne :</p>

<ul>
<li>Le support des requêtes Cypher pour l'exploration ad-hoc du graphe</li>
<li>L'accès serveur MCP pour que les agents IA puissent interroger le graphe</li>
<li>La correspondance rapide de patterns pour la découverte d'architecture</li>
</ul>

<p>Le choix architectural clé est que Kuzu est un <em>read model</em>, pas la source de vérité. La logique d'analyse ne doit pas être couplée aux mécaniques de stockage. Le graphe Rust et les artefacts JSON restent la représentation portable et canonique. Kuzu ajoute la puissance de requêtage sans créer de dépendance au stockage.</p>

<h2 id="what-this-enables">Ce que cela permet</h2>

<p>Avec un graphe typé, stratifié et préservant l'évidence, nous pouvons construire des détecteurs et des systèmes de gouvernance qui étaient auparavant impossibles :</p>

<ul>
<li><strong>Détection de cycles structurels</strong> qui ignore les cycles runtime intentionnels via les bus d'événements</li>
<li><strong>Détection de code mort</strong> qui comprend que les classes résolues par le framework ne sont pas véritablement mortes</li>
<li><strong>Identification de classes dieu</strong> qui prend en compte le couplage à différentes couches du graphe</li>
<li><strong>Génération de surface d'architecture</strong> qui montre aux développeurs où sont les vrais points de pression</li>
<li><strong>Revue alimentée par l'IA</strong> qui classe les résultats avec le contexte complet du graphe, pas seulement des heuristiques au niveau fichier</li>
</ul>

<p>C'est la différence entre un graphe de code bruyant et un système gardien utile. AigisCode n'essaie pas de compter les nœuds et les arêtes. Il essaie d'aider les humains et l'IA à comprendre comment un codebase est réellement câblé — où l'architecture est saine, où elle se dégrade, et que faire à ce sujet.</p>

<h2 id="try-it-yourself">Essayez par vous-même</h2>

<p>AigisCode est open source et sous licence MIT. Vous pouvez l'exécuter sur votre propre codebase dès aujourd'hui :</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
aigiscode analyze /path/to/your/project
</code></pre>

<p>L'analyse produit des artefacts JSON structurés dans <code>.aigiscode/</code> que tout agent IA ou pipeline CI peut consommer. Nous serions ravis de savoir à quoi ressemble votre graphe.</p>
`,
      es: `
<p>La mayoría de las herramientas de análisis estático tratan tu codebase como una bolsa de archivos. Escanean cada archivo de forma aislada, señalan violaciones de estilo y siguen adelante. Pero la verdadera arquitectura de software vive en las <strong>relaciones entre archivos</strong> — los imports, las llamadas, las cadenas de herencia, las suscripciones a eventos y los patrones de dispatch en runtime que conectan todo.</p>

<p>En AigisCode, estamos construyendo algo diferente: un <strong>grafo de código semántico</strong> que captura no solo qué depende de qué, sino <em>cómo</em>, <em>por qué</em> y <em>en qué capa</em> existen esas dependencias. Esta es la historia técnica de cómo llegamos aquí.</p>

<h2 id="why-flat-graphs-fail">Por qué fallan los grafos de código planos</h2>

<p>Un grafo de dependencias plano dice "el archivo A depende del archivo B." Eso es útil, pero limitado. Considera una aplicación Laravel donde un controlador llama a un servicio, que despacha un trabajo en cola, que resuelve un repositorio a través del contenedor IoC. En un grafo plano, ves cuatro nodos y tres aristas. En realidad, tres <em>tipos</em> diferentes de dependencia están en juego:</p>

<ul>
<li><strong>Estructural</strong> — la declaración <code>use</code> que importa la clase del servicio</li>
<li><strong>Runtime</strong> — el despacho de cola que conecta el trabajo en tiempo de ejecución</li>
<li><strong>Framework</strong> — la resolución del contenedor que gestiona el IoC</li>
</ul>

<p>Si aplanas los tres en el mismo tipo de arista, pierdes la capacidad de razonar sobre ellos de forma diferente. No puedes distinguir un ciclo estructural (siempre problemático) de un ciclo runtime a través del bus de eventos (a menudo intencional). No puedes saber si una clase "muerta" es verdaderamente inalcanzable o simplemente resuelta a través de una convención del framework que tu herramienta no entiende.</p>

<p>Este es el problema fundamental que nos propusimos resolver.</p>

<h2 id="the-canonical-rust-graph">El grafo canónico en Rust</h2>

<p>La fuente de verdad en AigisCode es un grafo semántico construido enteramente en Rust nativo. Elegimos Rust por las mismas razones por las que lo elegirías para cualquier sistema crítico en rendimiento: abstracciones de costo cero, seguridad de memoria sin recolector de basura y la capacidad de procesar codebases de más de 30,000 archivos en menos de 25 segundos.</p>

<p>Cada arista resuelta en nuestro grafo lleva metadatos tipados:</p>

<table>
<thead>
<tr><th>Campo</th><th>Propósito</th></tr>
</thead>
<tbody>
<tr><td><code>ReferenceKind</code></td><td>Qué tipo de referencia (llamada, import, extends, implements)</td></tr>
<tr><td><code>RelationKind</code></td><td>La relación semántica (dependencia, herencia, evento)</td></tr>
<tr><td><code>GraphLayer</code></td><td>Estructural, runtime, framework o policy-overlay</td></tr>
<tr><td><code>EdgeStrength</code></td><td>Qué tan fuerte es el acoplamiento</td></tr>
<tr><td><code>EdgeOrigin</code></td><td>Dónde se descubrió la arista (parser, resolver, plugin)</td></tr>
<tr><td><code>ResolutionTier</code></td><td>Qué tan confiable es la resolución</td></tr>
</tbody>
</table>

<p>Esto significa que cada arista no es solo "A depende de B" — es "A depende de B <em>a través de esta relación, en esta capa, con esta confianza, por esta razón</em>." Esa distinción es crítica para la explicabilidad y para el juicio arquitectónico basado en doctrina que estamos construyendo.</p>

<h2 id="layered-meaning">Significado estratificado</h2>

<p>Nos alejamos del conjunto de aristas plano temprano en el desarrollo. El modelo actual distingue cuatro capas:</p>

<ol>
<li><strong>Aristas estructurales</strong> — imports directos, referencias de clases, anotaciones de tipos</li>
<li><strong>Aristas runtime</strong> — despacho de colas, emisión de eventos, resolución dinámica</li>
<li><strong>Aristas framework</strong> — vinculaciones del contenedor IoC, hooks de WordPress, service providers de Laravel</li>
<li><strong>Aristas policy-overlay</strong> — aristas añadidas por reglas de configuración para convenciones aceptadas del codebase</li>
</ol>

<p>Esta estratificación nos permite hacer preguntas fundamentalmente diferentes contra diferentes vistas del grafo. Podemos detectar ciclos estructurales por separado de los ciclos expandidos por runtime. Podemos identificar artefactos del framework sin confundirlos con acoplamiento real. Y podemos permitir que los usuarios declaren qué patrones son intencionales a través de reglas de política, sin modificar el grafo central.</p>

<h2 id="plugin-expanded-framework-behavior">Comportamiento de framework expandido por plugins</h2>

<p>Una de nuestras decisiones arquitectónicas más importantes es que el conocimiento del framework no reside en los parsers de lenguaje centrales. En su lugar, reside en los <strong>plugins</strong>:</p>

<ul>
<li>El <strong>plugin de cola</strong> expande el despacho de trabajos en aristas runtime</li>
<li>El <strong>plugin de contenedor</strong> resuelve las vinculaciones IoC en aristas framework</li>
<li>El <strong>plugin de WordPress</strong> mapea <code>add_action</code> / <code>do_action</code> en aristas publish/subscribe</li>
</ul>

<p>El principio es simple:</p>

<ul>
<li>La verdad del lenguaje pertenece al núcleo</li>
<li>La verdad del framework pertenece a los plugins</li>
<li>El comportamiento aceptado específico del repositorio pertenece a las reglas de política</li>
</ul>

<p>Sin esta separación, el producto colapsaría en hacks específicos de repositorio. Cada instalación de WordPress requeriría patrones codificados diferentes. Cada versión de Laravel rompería el grafo. Al mantener el conocimiento del framework en plugins, podemos evolucionar el soporte de frameworks independientemente del motor de análisis central.</p>

<h2 id="two-views-one-truth">Dos vistas, una verdad</h2>

<p>La corrección más importante en nuestra última iteración fue separar el <strong>grafo canónico</strong> de la <strong>vista de dependencias</strong>.</p>

<p>Nuestras primeras exportaciones del grafo eran demasiado ruidosas. Incluían nodos MODULE sintéticos para cada archivo, aristas CONTAINS para cada símbolo y aristas de sitios de llamada repetidos contados individualmente. Esto hacía que el grafo pareciera impresionantemente grande, pero gran parte de ese tamaño era sobrecarga representacional, no valor arquitectónico.</p>

<p>Ahora mantenemos dos vistas desde la misma fuente de verdad:</p>

<h3 id="canonical-graph">Grafo canónico (optimizado para evidencia)</h3>
<p>El grafo canónico retiene todo: sitios de llamada repetidos, aristas runtime y de plugin detalladas, información semántica de grano fino y toda la evidencia necesaria para investigación profunda. Esto es lo que alimenta nuestros detectores y la etapa de revisión por IA.</p>

<h3 id="dependency-view">Vista de dependencias (optimizada para consultas)</h3>
<p>La vista de dependencias es una proyección normalizada que omite nodos sintéticos, omite aristas de contención, remapea aristas dirigidas a módulos sobre nodos de archivo y colapsa dependencias repetidas en una sola arista con un <code>occurrenceCount</code>. Esto es lo que alimenta nuestro reporting, acceso MCP y exploración de arquitectura.</p>

<p>En otras palabras: el grafo canónico optimiza para la verdad y la evidencia. La vista de dependencias optimiza para una interpretación arquitectónica de bajo ruido.</p>

<h2 id="wordpress-benchmark">Benchmark de WordPress: 32,862 nodos en 22.78 segundos</h2>

<p>Hacemos benchmark contra WordPress — uno de los codebases PHP más grandes y complejos del mundo. Aquí están nuestros números actuales de la vista de dependencias normalizada:</p>

<table>
<thead>
<tr><th>Métrica</th><th>AigisCode</th></tr>
</thead>
<tbody>
<tr><td>Tiempo total</td><td>22.78s</td></tr>
<tr><td>Nodos</td><td>32,862</td></tr>
<tr><td>Relaciones</td><td>95,878</td></tr>
</tbody>
</table>

<p>El desglose de relaciones revela la riqueza de nuestro grafo:</p>

<ul>
<li><strong>CALL</strong>: 85,451 — invocaciones de funciones y métodos</li>
<li><strong>EVENTPUBLISH</strong>: 3,662 — activaciones de hooks de WordPress (<code>do_action</code>, <code>apply_filters</code>)</li>
<li><strong>OVERRIDES</strong>: 1,947 — sobreescrituras de métodos en jerarquías de clases</li>
<li><strong>EVENTSUBSCRIBE</strong>: 1,868 — registros de hooks (<code>add_action</code>, <code>add_filter</code>)</li>
<li><strong>EXTENDS</strong>: 1,489 — herencia de clases</li>
<li><strong>IMPORT</strong>: 764 — imports e inclusiones a nivel de archivo</li>
<li><strong>TYPEUSE</strong>: 625 — anotaciones e indicaciones de tipos</li>
<li><strong>IMPLEMENTS</strong>: 72 — implementaciones de interfaces</li>
</ul>

<p>Las aristas de hooks de WordPress (EVENTPUBLISH + EVENTSUBSCRIBE) son particularmente significativas. Representan cableado runtime que las herramientas de análisis estático planas pasan completamente por alto. Cuando WordPress llama a <code>do_action('init')</code>, 47 plugins diferentes responden. Nuestro grafo captura las 47 conexiones.</p>

<h2 id="optional-kuzu-read-model">Read model Kuzu opcional</h2>

<p>Para consultas y exploración, opcionalmente exportamos la vista de dependencias a <a href="https://kuzudb.com/">Kuzu</a>, una base de datos de grafos embebida. Esto nos da:</p>

<ul>
<li>Soporte de consultas Cypher para exploración ad-hoc del grafo</li>
<li>Acceso al servidor MCP para que los agentes de IA consulten el grafo</li>
<li>Coincidencia rápida de patrones para descubrimiento de arquitectura</li>
</ul>

<p>La decisión arquitectónica clave es que Kuzu es un <em>read model</em>, no la fuente de verdad. La lógica de análisis no debería estar acoplada a la mecánica de almacenamiento. El grafo Rust y los artefactos JSON siguen siendo la representación portable y canónica. Kuzu añade poder de consulta sin crear dependencia de almacenamiento.</p>

<h2 id="what-this-enables">Qué permite esto</h2>

<p>Con un grafo tipado, estratificado y que preserva la evidencia, podemos construir detectores y sistemas de gobernanza que antes eran imposibles:</p>

<ul>
<li><strong>Detección de ciclos estructurales</strong> que ignora los ciclos runtime intencionales a través de buses de eventos</li>
<li><strong>Detección de código muerto</strong> que entiende que las clases resueltas por el framework no están verdaderamente muertas</li>
<li><strong>Identificación de clases dios</strong> que tiene en cuenta el acoplamiento en diferentes capas del grafo</li>
<li><strong>Generación de superficie arquitectónica</strong> que muestra a los desarrolladores dónde están los verdaderos puntos de presión</li>
<li><strong>Revisión potenciada por IA</strong> que clasifica hallazgos con el contexto completo del grafo, no solo heurísticas a nivel de archivo</li>
</ul>

<p>Esta es la diferencia entre un grafo de código ruidoso y un sistema guardián útil. AigisCode no intenta contar nodos y aristas. Intenta ayudar a los humanos y a la IA a entender cómo está realmente cableado un codebase — dónde la arquitectura es saludable, dónde se está degradando y qué hacer al respecto.</p>

<h2 id="try-it-yourself">Pruébalo tú mismo</h2>

<p>AigisCode es open source y tiene licencia MIT. Puedes ejecutarlo en tu propio codebase hoy:</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
aigiscode analyze /path/to/your/project
</code></pre>

<p>El análisis produce artefactos JSON estructurados en <code>.aigiscode/</code> que cualquier agente de IA o pipeline de CI puede consumir. Nos encantaría saber cómo se ve tu grafo.</p>
`,
      ar: `
<p>تتعامل معظم أدوات التحليل الثابت مع قاعدة الكود الخاصة بك كمجموعة من الملفات. تفحص كل ملف بمعزل عن غيره، وتضع علامات على مخالفات الأنماط، ثم تنتقل للملف التالي. لكن هندسة البرمجيات الحقيقية تعيش في <strong>العلاقات بين الملفات</strong> — الاستيرادات والاستدعاءات وسلاسل الوراثة واشتراكات الأحداث وأنماط الإرسال في وقت التشغيل التي تربط كل شيء معاً.</p>

<p>في AigisCode، نبني شيئاً مختلفاً: <strong>رسماً بيانياً دلالياً للكود</strong> لا يلتقط فقط ما يعتمد على ماذا، بل <em>كيف</em> و<em>لماذا</em> و<em>في أي طبقة</em> توجد تلك الاعتماديات. هذه هي القصة التقنية لكيفية وصولنا إلى هنا.</p>

<h2 id="why-flat-graphs-fail">لماذا تفشل الرسوم البيانية المسطحة للكود</h2>

<p>يقول الرسم البياني المسطح للاعتماديات "الملف A يعتمد على الملف B." هذا مفيد، لكنه محدود. فكر في تطبيق Laravel حيث يستدعي المتحكم خدمة، والتي ترسل مهمة مُصفّة، والتي تحل مستودعاً من خلال حاوية IoC. في رسم بياني مسطح، ترى أربع عقد وثلاث حواف. في الواقع، ثلاثة <em>أنواع</em> مختلفة من الاعتماديات تعمل:</p>

<ul>
<li><strong>هيكلية</strong> — عبارة <code>use</code> التي تستورد فئة الخدمة</li>
<li><strong>وقت التشغيل</strong> — إرسال قائمة الانتظار الذي يربط المهمة في وقت التشغيل</li>
<li><strong>إطار العمل</strong> — تحليل الحاوية الذي يديره IoC</li>
</ul>

<p>إذا قمت بتسطيح الثلاثة في نفس نوع الحافة، تفقد القدرة على التفكير فيها بشكل مختلف. لا يمكنك التمييز بين دورة هيكلية (دائماً مشكلة) ودورة وقت تشغيل عبر ناقل الأحداث (غالباً مقصودة). لا يمكنك معرفة ما إذا كانت فئة "ميتة" غير قابلة للوصول حقاً أم يتم حلها ببساطة من خلال اصطلاح إطار عمل لا تفهمه أداتك.</p>

<p>هذه هي المشكلة الأساسية التي سعينا لحلها.</p>

<h2 id="the-canonical-rust-graph">الرسم البياني الأساسي في Rust</h2>

<p>مصدر الحقيقة في AigisCode هو رسم بياني دلالي مبني بالكامل في Rust الأصلي. اخترنا Rust لنفس الأسباب التي تجعلك تختاره لأي نظام حساس للأداء: تجريدات بدون تكلفة، وأمان الذاكرة بدون جامع نفايات، والقدرة على معالجة أكثر من 30,000 ملف في أقل من 25 ثانية.</p>

<p>كل حافة محلولة في رسمنا البياني تحمل بيانات وصفية مصنّفة:</p>

<table>
<thead>
<tr><th>الحقل</th><th>الغرض</th></tr>
</thead>
<tbody>
<tr><td><code>ReferenceKind</code></td><td>نوع المرجع (استدعاء، استيراد، يمتد، ينفّذ)</td></tr>
<tr><td><code>RelationKind</code></td><td>العلاقة الدلالية (اعتمادية، وراثة، حدث)</td></tr>
<tr><td><code>GraphLayer</code></td><td>هيكلية، وقت تشغيل، إطار عمل، أو تراكب سياسة</td></tr>
<tr><td><code>EdgeStrength</code></td><td>مدى قوة الترابط</td></tr>
<tr><td><code>EdgeOrigin</code></td><td>أين تم اكتشاف الحافة (محلل، محلل أسماء، إضافة)</td></tr>
<tr><td><code>ResolutionTier</code></td><td>مدى الثقة في الحل</td></tr>
</tbody>
</table>

<p>هذا يعني أن كل حافة ليست مجرد "A يعتمد على B" — بل "A يعتمد على B <em>من خلال هذه العلاقة، في هذه الطبقة، بهذه الثقة، لهذا السبب</em>." هذا التمييز ضروري للشرح وللحكم المعماري القائم على العقيدة الذي نبنيه.</p>

<h2 id="layered-meaning">المعنى المتعدد الطبقات</h2>

<p>ابتعدنا عن مجموعة الحواف المسطحة في وقت مبكر من التطوير. النموذج الحالي يميز أربع طبقات:</p>

<ol>
<li><strong>الحواف الهيكلية</strong> — الاستيرادات المباشرة، مراجع الفئات، التعليقات التوضيحية للأنواع</li>
<li><strong>حواف وقت التشغيل</strong> — إرسال قائمة الانتظار، إصدار الأحداث، الحل الديناميكي</li>
<li><strong>حواف إطار العمل</strong> — ربط حاوية IoC، خطافات WordPress، مزودو خدمة Laravel</li>
<li><strong>حواف تراكب السياسة</strong> — حواف تُضاف بقواعد التكوين لاصطلاحات قاعدة الكود المقبولة</li>
</ol>

<p>هذا التطبيق المتعدد الطبقات يتيح لنا طرح أسئلة مختلفة جوهرياً على عروض مختلفة للرسم البياني. يمكننا اكتشاف الدورات الهيكلية بشكل منفصل عن الدورات الموسعة في وقت التشغيل. يمكننا تحديد مكونات إطار العمل دون الخلط بينها وبين الترابط الحقيقي. ويمكننا السماح للمستخدمين بالإعلان عن الأنماط المقصودة من خلال قواعد السياسة، دون تعديل الرسم البياني الأساسي.</p>

<h2 id="plugin-expanded-framework-behavior">سلوك إطار العمل الموسّع بالإضافات</h2>

<p>أحد أهم قراراتنا المعمارية هو أن معرفة إطار العمل لا تعيش في محللات اللغة الأساسية. بدلاً من ذلك، تعيش في <strong>الإضافات</strong>:</p>

<ul>
<li><strong>إضافة قائمة الانتظار</strong> توسع إرسال المهام إلى حواف وقت التشغيل</li>
<li><strong>إضافة الحاوية</strong> تحل ربط IoC إلى حواف إطار العمل</li>
<li><strong>إضافة WordPress</strong> تحول <code>add_action</code> / <code>do_action</code> إلى حواف نشر/اشتراك</li>
</ul>

<p>المبدأ بسيط:</p>

<ul>
<li>حقيقة اللغة تنتمي إلى النواة</li>
<li>حقيقة إطار العمل تنتمي إلى الإضافات</li>
<li>السلوك المقبول الخاص بالمستودع ينتمي إلى قواعد السياسة</li>
</ul>

<p>بدون هذا الفصل، سينهار المنتج في اختراقات خاصة بالمستودع. كل تثبيت WordPress سيتطلب أنماطاً ثابتة مختلفة. كل إصدار من Laravel سيكسر الرسم البياني. من خلال الحفاظ على معرفة إطار العمل في الإضافات، يمكننا تطوير دعم إطار العمل بشكل مستقل عن محرك التحليل الأساسي.</p>

<h2 id="two-views-one-truth">عرضان، حقيقة واحدة</h2>

<p>أهم تصحيح في آخر تكرار لدينا كان فصل <strong>الرسم البياني الأساسي</strong> عن <strong>عرض الاعتماديات</strong>.</p>

<p>كانت صادرات رسمنا البياني الأولية صاخبة جداً. تضمنت عقد MODULE اصطناعية لكل ملف، وحواف CONTAINS لكل رمز، وحواف مواقع الاستدعاء المكررة المحسوبة بشكل فردي. هذا جعل الرسم البياني يبدو كبيراً بشكل مثير للإعجاب، لكن الكثير من هذا الحجم كان عبئاً تمثيلياً، وليس قيمة معمارية.</p>

<p>نحتفظ الآن بعرضين من نفس مصدر الحقيقة:</p>

<h3 id="canonical-graph">الرسم البياني الأساسي (محسّن للأدلة)</h3>
<p>يحتفظ الرسم البياني الأساسي بكل شيء: مواقع الاستدعاء المكررة، وحواف وقت التشغيل والإضافات المفصلة، والمعلومات الدلالية الدقيقة، وجميع الأدلة اللازمة للتحقيق العميق. هذا ما يشغّل كاشفاتنا ومرحلة مراجعة الذكاء الاصطناعي.</p>

<h3 id="dependency-view">عرض الاعتماديات (محسّن للاستعلام)</h3>
<p>عرض الاعتماديات هو إسقاط طبيعي يحذف العقد الاصطناعية، ويحذف حواف الاحتواء، ويعيد تعيين الحواف المستهدفة للوحدات على عقد الملفات، ويطوي الاعتماديات المكررة في حافة واحدة مع <code>occurrenceCount</code>. هذا ما يشغّل تقاريرنا ووصول MCP واستكشاف البنية المعمارية.</p>

<p>بعبارة أخرى: الرسم البياني الأساسي يحسّن للحقيقة والأدلة. عرض الاعتماديات يحسّن للتفسير المعماري منخفض الضوضاء.</p>

<h2 id="wordpress-benchmark">معيار WordPress: 32,862 عقدة في 22.78 ثانية</h2>

<p>نقيس الأداء مقابل WordPress — واحدة من أكبر وأعقد قواعد كود PHP في العالم. هذه أرقامنا الحالية من عرض الاعتماديات الطبيعي:</p>

<table>
<thead>
<tr><th>المقياس</th><th>AigisCode</th></tr>
</thead>
<tbody>
<tr><td>الوقت الكلي</td><td>22.78s</td></tr>
<tr><td>العقد</td><td>32,862</td></tr>
<tr><td>العلاقات</td><td>95,878</td></tr>
</tbody>
</table>

<p>تفصيل العلاقات يكشف عن غنى رسمنا البياني:</p>

<ul>
<li><strong>CALL</strong>: 85,451 — استدعاءات الدوال والأساليب</li>
<li><strong>EVENTPUBLISH</strong>: 3,662 — تفعيلات خطافات WordPress (<code>do_action</code>، <code>apply_filters</code>)</li>
<li><strong>OVERRIDES</strong>: 1,947 — تجاوزات الأساليب في تسلسلات الفئات</li>
<li><strong>EVENTSUBSCRIBE</strong>: 1,868 — تسجيلات الخطافات (<code>add_action</code>، <code>add_filter</code>)</li>
<li><strong>EXTENDS</strong>: 1,489 — وراثة الفئات</li>
<li><strong>IMPORT</strong>: 764 — استيرادات وتضمينات على مستوى الملف</li>
<li><strong>TYPEUSE</strong>: 625 — التعليقات التوضيحية والتلميحات للأنواع</li>
<li><strong>IMPLEMENTS</strong>: 72 — تنفيذات الواجهات</li>
</ul>

<p>حواف خطافات WordPress (EVENTPUBLISH + EVENTSUBSCRIBE) ذات أهمية خاصة. تمثل هذه الربط في وقت التشغيل الذي تفتقده أدوات التحليل الثابت المسطحة تماماً. عندما يستدعي WordPress <code>do_action('init')</code>، تستجيب 47 إضافة مختلفة. رسمنا البياني يلتقط جميع هذه الاتصالات الـ 47.</p>

<h2 id="optional-kuzu-read-model">نموذج القراءة الاختياري Kuzu</h2>

<p>للاستعلام والاستكشاف، نصدّر اختيارياً عرض الاعتماديات إلى <a href="https://kuzudb.com/">Kuzu</a>، قاعدة بيانات رسوم بيانية مضمنة. يمنحنا هذا:</p>

<ul>
<li>دعم استعلامات Cypher لاستكشاف الرسم البياني المخصص</li>
<li>وصول خادم MCP لوكلاء الذكاء الاصطناعي للاستعلام عن الرسم البياني</li>
<li>مطابقة أنماط سريعة لاكتشاف البنية المعمارية</li>
</ul>

<p>الخيار المعماري الرئيسي هو أن Kuzu هو <em>نموذج قراءة</em>، وليس مصدر الحقيقة. لا ينبغي أن يكون منطق التحليل مرتبطاً بآليات التخزين. يبقى رسم Rust البياني ومخرجات JSON هي التمثيل الأساسي المحمول. يضيف Kuzu قوة الاستعلام دون إنشاء اعتماد على التخزين.</p>

<h2 id="what-this-enables">ما الذي يتيحه هذا</h2>

<p>مع رسم بياني مصنّف ومتعدد الطبقات ويحافظ على الأدلة، يمكننا بناء كاشفات وأنظمة حوكمة كانت مستحيلة سابقاً:</p>

<ul>
<li><strong>كشف الدورات الهيكلية</strong> الذي يتجاهل دورات وقت التشغيل المقصودة عبر ناقلات الأحداث</li>
<li><strong>كشف الكود الميت</strong> الذي يفهم أن الفئات المحلولة بإطار العمل ليست ميتة حقاً</li>
<li><strong>تحديد الفئات العملاقة</strong> الذي يأخذ في الاعتبار الترابط في طبقات مختلفة من الرسم البياني</li>
<li><strong>إنشاء سطح البنية المعمارية</strong> الذي يُظهر للمطورين أين تقع نقاط الضغط الحقيقية</li>
<li><strong>مراجعة مدعومة بالذكاء الاصطناعي</strong> تصنف النتائج بسياق الرسم البياني الكامل، وليس مجرد استدلالات على مستوى الملف</li>
</ul>

<p>هذا هو الفرق بين رسم بياني صاخب للكود ونظام حماية مفيد. لا يحاول AigisCode عدّ العقد والحواف. بل يحاول مساعدة البشر والذكاء الاصطناعي على فهم كيفية ربط قاعدة الكود فعلياً — أين تكون البنية المعمارية صحية، وأين تتدهور، وما الذي يجب فعله حيال ذلك.</p>

<h2 id="try-it-yourself">جرّبه بنفسك</h2>

<p>AigisCode مفتوح المصدر ومرخص تحت MIT. يمكنك تشغيله على قاعدة الكود الخاصة بك اليوم:</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
aigiscode analyze /path/to/your/project
</code></pre>

<p>ينتج التحليل مخرجات JSON منظمة في <code>.aigiscode/</code> يمكن لأي وكيل ذكاء اصطناعي أو خط أنابيب CI استهلاكها. يسعدنا أن نسمع كيف يبدو رسمك البياني.</p>
`,
      pl: `
<p>Większość narzędzi do analizy statycznej traktuje Twoją bazę kodu jak worek plików. Skanują każdy plik w izolacji, oznaczają naruszenia stylu i przechodzą dalej. Ale prawdziwa architektura oprogramowania żyje w <strong>relacjach między plikami</strong> — importach, wywołaniach, łańcuchach dziedziczenia, subskrypcjach zdarzeń i wzorcach dispatch w runtime, które łączą wszystko razem.</p>

<p>W AigisCode budujemy coś innego: <strong>semantyczny graf kodu</strong>, który rejestruje nie tylko co od czego zależy, ale <em>jak</em>, <em>dlaczego</em> i <em>na jakiej warstwie</em> te zależności istnieją. To jest techniczna historia tego, jak tu dotarliśmy.</p>

<h2 id="why-flat-graphs-fail">Dlaczego płaskie grafy kodu zawodzą</h2>

<p>Płaski graf zależności mówi "plik A zależy od pliku B." To jest przydatne, ale ograniczone. Rozważ aplikację Laravel, gdzie kontroler wywołuje serwis, który dispatchuje zadanie kolejki, które rozwiązuje repozytorium przez kontener IoC. W płaskim grafie widzisz cztery węzły i trzy krawędzie. W rzeczywistości trzy różne <em>rodzaje</em> zależności są w grze:</p>

<ul>
<li><strong>Strukturalne</strong> — instrukcja <code>use</code> importująca klasę serwisu</li>
<li><strong>Runtime</strong> — dispatch kolejki, który wiąże zadanie w runtime</li>
<li><strong>Framework</strong> — rozwiązywanie kontenera, którym zarządza IoC</li>
</ul>

<p>Jeśli spłaszczysz wszystkie trzy do tego samego typu krawędzi, tracisz zdolność rozumowania o nich w różny sposób. Nie możesz odróżnić cyklu strukturalnego (zawsze problematyczny) od cyklu runtime przez magistralę zdarzeń (często zamierzony). Nie możesz stwierdzić, czy "martwa" klasa jest naprawdę nieosiągalna, czy po prostu rozwiązywana przez konwencję frameworka, której Twoje narzędzie nie rozumie.</p>

<p>To jest fundamentalny problem, który postanowiliśmy rozwiązać.</p>

<h2 id="the-canonical-rust-graph">Kanoniczny graf Rust</h2>

<p>Źródłem prawdy w AigisCode jest semantyczny graf zbudowany w całości w natywnym Rust. Wybraliśmy Rust z tych samych powodów, dla których wybrałbyś go do dowolnego systemu krytycznego pod względem wydajności: abstrakcje zero-cost, bezpieczeństwo pamięci bez garbage collectora i zdolność przetwarzania ponad 30 000 plików w mniej niż 25 sekund.</p>

<p>Każda rozwiązana krawędź w naszym grafie niesie typowane metadane:</p>

<table>
<thead>
<tr><th>Pole</th><th>Cel</th></tr>
</thead>
<tbody>
<tr><td><code>ReferenceKind</code></td><td>Rodzaj referencji (wywołanie, import, extends, implements)</td></tr>
<tr><td><code>RelationKind</code></td><td>Relacja semantyczna (zależność, dziedziczenie, zdarzenie)</td></tr>
<tr><td><code>GraphLayer</code></td><td>Strukturalna, runtime, framework lub nakładka polityk</td></tr>
<tr><td><code>EdgeStrength</code></td><td>Jak silne jest powiązanie</td></tr>
<tr><td><code>EdgeOrigin</code></td><td>Gdzie krawędź została odkryta (parser, resolver, plugin)</td></tr>
<tr><td><code>ResolutionTier</code></td><td>Jak pewne jest rozwiązanie</td></tr>
</tbody>
</table>

<p>To oznacza, że każda krawędź to nie tylko "A zależy od B" — to "A zależy od B <em>przez tę relację, na tej warstwie, z tą pewnością, z tego powodu</em>." To rozróżnienie jest krytyczne dla wyjaśniania i dla opartego na doktrynie osądu architektonicznego, który budujemy.</p>

<h2 id="layered-meaning">Warstwowe znaczenie</h2>

<p>Odeszliśmy od płaskiego zbioru krawędzi na wczesnym etapie rozwoju. Obecny model rozróżnia cztery warstwy:</p>

<ol>
<li><strong>Krawędzie strukturalne</strong> — bezpośrednie importy, referencje klas, adnotacje typów</li>
<li><strong>Krawędzie runtime</strong> — dispatch kolejki, emisja zdarzeń, dynamiczne rozwiązywanie</li>
<li><strong>Krawędzie framework</strong> — bindowania kontenera IoC, hooki WordPress, service providerzy Laravel</li>
<li><strong>Krawędzie nakładki polityk</strong> — krawędzie dodane przez reguły konfiguracji dla zaakceptowanych konwencji bazy kodu</li>
</ol>

<p>To warstwowanie pozwala nam zadawać zasadniczo różne pytania wobec różnych widoków grafu. Możemy wykrywać cykle strukturalne oddzielnie od cykli rozszerzonych w runtime. Możemy identyfikować artefakty frameworkowe bez mylenia ich z rzeczywistym powiązaniem. I możemy pozwolić użytkownikom deklarować, które wzorce są zamierzone poprzez reguły polityk, bez modyfikowania rdzenia grafu.</p>

<h2 id="plugin-expanded-framework-behavior">Zachowanie frameworka rozszerzone przez pluginy</h2>

<p>Jedną z naszych najważniejszych decyzji architektonicznych jest to, że wiedza o frameworku nie żyje w rdzeniowych parserach językowych. Zamiast tego żyje w <strong>pluginach</strong>:</p>

<ul>
<li><strong>Plugin kolejki</strong> rozszerza dispatch zadań na krawędzie runtime</li>
<li><strong>Plugin kontenera</strong> rozwiązuje bindowania IoC na krawędzie frameworka</li>
<li><strong>Plugin WordPress</strong> mapuje <code>add_action</code> / <code>do_action</code> na krawędzie publish/subscribe</li>
</ul>

<p>Zasada jest prosta:</p>

<ul>
<li>Prawda językowa należy do rdzenia</li>
<li>Prawda frameworkowa należy do pluginów</li>
<li>Zaakceptowane zachowanie specyficzne dla repozytorium należy do reguł polityk</li>
</ul>

<p>Bez tego rozdziału produkt rozpadłby się na hacki specyficzne dla repozytorium. Każda instalacja WordPress wymagałaby innych zakodowanych na sztywno wzorców. Każda wersja Laravel łamałaby graf. Utrzymując wiedzę o frameworku w pluginach, możemy rozwijać wsparcie frameworków niezależnie od rdzeniowego silnika analizy.</p>

<h2 id="two-views-one-truth">Dwa widoki, jedna prawda</h2>

<p>Najważniejsza korekta w naszej ostatniej iteracji było oddzielenie <strong>kanonicznego grafu</strong> od <strong>widoku zależności</strong>.</p>

<p>Nasze początkowe eksporty grafu były zbyt zaszumione. Zawierały syntetyczne węzły MODULE dla każdego pliku, krawędzie CONTAINS dla każdego symbolu i powtarzane krawędzie miejsc wywołań liczone indywidualnie. To sprawiło, że graf wyglądał imponująco duży, ale większość tego rozmiaru to narzut reprezentacyjny, nie wartość architektoniczna.</p>

<p>Teraz utrzymujemy dwa widoki z tego samego źródła prawdy:</p>

<h3 id="canonical-graph">Graf kanoniczny (zoptymalizowany pod kątem dowodów)</h3>
<p>Graf kanoniczny zachowuje wszystko: powtarzane miejsca wywołań, szczegółowe krawędzie runtime i pluginów, drobnoziarniste informacje semantyczne i wszystkie dowody potrzebne do głębokiego dochodzenia. To napędza nasze detektory i etap przeglądu AI.</p>

<h3 id="dependency-view">Widok zależności (zoptymalizowany pod kątem zapytań)</h3>
<p>Widok zależności to znormalizowana projekcja, która pomija syntetyczne węzły, pomija krawędzie zawierania, przemapowuje krawędzie ukierunkowane na moduły na węzły plików i składa powtarzane zależności w pojedynczą krawędź z <code>occurrenceCount</code>. To napędza nasze raportowanie, dostęp MCP i eksplorację architektury.</p>

<p>Innymi słowy: graf kanoniczny optymalizuje pod kątem prawdy i dowodów. Widok zależności optymalizuje pod kątem interpretacji architektonicznej o niskim szumie.</p>

<h2 id="wordpress-benchmark">Benchmark WordPress: 32 862 węzły w 22,78 sekund</h2>

<p>Benchmarkujemy na WordPress — jednej z największych i najbardziej złożonych baz kodu PHP na świecie. Oto nasze obecne liczby ze znormalizowanego widoku zależności:</p>

<table>
<thead>
<tr><th>Metryka</th><th>AigisCode</th></tr>
</thead>
<tbody>
<tr><td>Czas zegarowy</td><td>22.78s</td></tr>
<tr><td>Węzły</td><td>32,862</td></tr>
<tr><td>Relacje</td><td>95,878</td></tr>
</tbody>
</table>

<p>Rozkład relacji ukazuje bogactwo naszego grafu:</p>

<ul>
<li><strong>CALL</strong>: 85,451 — wywołania funkcji i metod</li>
<li><strong>EVENTPUBLISH</strong>: 3,662 — aktywacje hooków WordPress (<code>do_action</code>, <code>apply_filters</code>)</li>
<li><strong>OVERRIDES</strong>: 1,947 — nadpisania metod w hierarchiach klas</li>
<li><strong>EVENTSUBSCRIBE</strong>: 1,868 — rejestracje hooków (<code>add_action</code>, <code>add_filter</code>)</li>
<li><strong>EXTENDS</strong>: 1,489 — dziedziczenie klas</li>
<li><strong>IMPORT</strong>: 764 — importy i includy na poziomie plików</li>
<li><strong>TYPEUSE</strong>: 625 — adnotacje typów i podpowiedzi</li>
<li><strong>IMPLEMENTS</strong>: 72 — implementacje interfejsów</li>
</ul>

<p>Krawędzie hooków WordPress (EVENTPUBLISH + EVENTSUBSCRIBE) są szczególnie istotne. Reprezentują wiązanie w runtime, które płaskie narzędzia analizy statycznej całkowicie pomijają. Gdy WordPress wywołuje <code>do_action('init')</code>, odpowiada 47 różnych pluginów. Nasz graf rejestruje wszystkie 47 tych połączeń.</p>

<h2 id="optional-kuzu-read-model">Opcjonalny model odczytu Kuzu</h2>

<p>Do zapytań i eksploracji opcjonalnie eksportujemy widok zależności do <a href="https://kuzudb.com/">Kuzu</a>, osadzonej bazy danych grafowej. Daje nam to:</p>

<ul>
<li>Wsparcie zapytań Cypher do ad-hoc eksploracji grafu</li>
<li>Dostęp serwera MCP dla agentów AI do odpytywania grafu</li>
<li>Szybkie dopasowywanie wzorców do odkrywania architektury</li>
</ul>

<p>Kluczowy wybór architektoniczny polega na tym, że Kuzu jest <em>modelem odczytu</em>, a nie źródłem prawdy. Logika analizy nie powinna być sprzęgnięta z mechaniką przechowywania. Graf Rust i artefakty JSON pozostają przenośną, kanoniczną reprezentacją. Kuzu dodaje moc zapytań bez tworzenia zależności od przechowywania.</p>

<h2 id="what-this-enables">Co to umożliwia</h2>

<p>Z typowanym, warstwowym, zachowującym dowody grafem możemy budować detektory i systemy zarządzania, które wcześniej były niemożliwe:</p>

<ul>
<li><strong>Detekcja cykli strukturalnych</strong>, która ignoruje zamierzone cykle runtime przez magistralę zdarzeń</li>
<li><strong>Detekcja martwego kodu</strong>, która rozumie, że klasy rozwiązywane przez framework nie są naprawdę martwe</li>
<li><strong>Identyfikacja god klas</strong>, która uwzględnia powiązanie na różnych warstwach grafu</li>
<li><strong>Generowanie powierzchni architektury</strong>, które pokazuje deweloperom, gdzie są prawdziwe punkty nacisku</li>
<li><strong>Przegląd wspomagany AI</strong>, który klasyfikuje znaleziska z pełnym kontekstem grafu, nie tylko heurystykami na poziomie pliku</li>
</ul>

<p>To jest różnica między zaszumionym grafem kodu a użytecznym systemem strażniczym. AigisCode nie próbuje liczyć węzłów i krawędzi. Próbuje pomóc ludziom i AI zrozumieć, jak baza kodu jest naprawdę połączona — gdzie architektura jest zdrowa, gdzie się degraduje i co z tym zrobić.</p>

<h2 id="try-it-yourself">Wypróbuj sam</h2>

<p>AigisCode jest open source i na licencji MIT. Możesz uruchomić go na swojej bazie kodu już dziś:</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
aigiscode analyze /path/to/your/project
</code></pre>

<p>Analiza produkuje ustrukturyzowane artefakty JSON w <code>.aigiscode/</code>, które dowolny agent AI lub pipeline CI może skonsumować. Chętnie usłyszymy, jak wygląda Twój graf.</p>
`,
      bn: `
<p>বেশিরভাগ স্ট্যাটিক অ্যানালিসিস টুল আপনার কোডবেসকে একটি ফাইলের থলি হিসেবে বিবেচনা করে। তারা প্রতিটি ফাইল আলাদাভাবে স্ক্যান করে, স্টাইল লঙ্ঘন চিহ্নিত করে এবং পরবর্তীতে চলে যায়। কিন্তু প্রকৃত সফটওয়্যার আর্কিটেকচার <strong>ফাইলগুলোর মধ্যকার সম্পর্কে</strong> বাস করে — ইমপোর্ট, কল, ইনহেরিট্যান্স চেইন, ইভেন্ট সাবস্ক্রিপশন এবং রানটাইম ডিসপ্যাচ প্যাটার্ন যা সবকিছুকে একসাথে সংযুক্ত করে।</p>

<p>AigisCode-এ, আমরা ভিন্ন কিছু তৈরি করছি: একটি <strong>সিমান্টিক কোড গ্রাফ</strong> যা শুধু কী কিসের উপর নির্ভর করে তা নয়, বরং <em>কিভাবে</em>, <em>কেন</em>, এবং <em>কোন লেয়ারে</em> সেই ডিপেন্ডেন্সিগুলো বিদ্যমান তাও ক্যাপচার করে। এটি আমরা এখানে কিভাবে পৌঁছালাম তার প্রযুক্তিগত গল্প।</p>

<h2 id="why-flat-graphs-fail">ফ্ল্যাট কোড গ্রাফ কেন ব্যর্থ হয়</h2>

<p>একটি ফ্ল্যাট ডিপেন্ডেন্সি গ্রাফ বলে "ফাইল A ফাইল B-র উপর নির্ভর করে।" এটি দরকারী, কিন্তু সীমিত। একটি Laravel অ্যাপ্লিকেশন বিবেচনা করুন যেখানে একটি কন্ট্রোলার একটি সার্ভিস কল করে, যা একটি কিউড জব ডিসপ্যাচ করে, যা IoC কন্টেইনারের মাধ্যমে একটি রিপোজিটরি রিজলভ করে। একটি ফ্ল্যাট গ্রাফে, আপনি চারটি নোড এবং তিনটি এজ দেখেন। বাস্তবে, তিনটি ভিন্ন <em>ধরনের</em> ডিপেন্ডেন্সি কাজ করছে:</p>

<ul>
<li><strong>স্ট্রাকচারাল</strong> — সার্ভিস ক্লাস ইমপোর্ট করা <code>use</code> স্টেটমেন্ট</li>
<li><strong>রানটাইম</strong> — কিউ ডিসপ্যাচ যা রানটাইমে জব ওয়্যার করে</li>
<li><strong>ফ্রেমওয়ার্ক</strong> — IoC পরিচালিত কন্টেইনার রেজোলিউশন</li>
</ul>

<p>যদি আপনি তিনটিকে একই এজ টাইপে ফ্ল্যাট করেন, আপনি তাদের সম্পর্কে আলাদাভাবে রিজন করার ক্ষমতা হারান। আপনি একটি স্ট্রাকচারাল সাইকেল (সর্বদা সমস্যাজনক) এবং ইভেন্ট বাসের মাধ্যমে একটি রানটাইম সাইকেল (প্রায়ই ইচ্ছাকৃত) আলাদা করতে পারেন না। আপনি বলতে পারেন না একটি "ডেড" ক্লাস সত্যিই অপ্রাপ্য কিনা বা শুধু একটি ফ্রেমওয়ার্ক কনভেনশনের মাধ্যমে রিজলভ হচ্ছে যা আপনার টুল বোঝে না।</p>

<p>এটিই সেই মৌলিক সমস্যা যা আমরা সমাধান করতে বেরিয়েছিলাম।</p>

<h2 id="the-canonical-rust-graph">ক্যানোনিক্যাল Rust গ্রাফ</h2>

<p>AigisCode-এ সত্যের উৎস হলো নেটিভ Rust-এ সম্পূর্ণভাবে নির্মিত একটি সিমান্টিক গ্রাফ। আমরা Rust বেছে নিয়েছি সেই একই কারণে যে কারণে আপনি এটি যেকোনো পারফরম্যান্স-ক্রিটিক্যাল সিস্টেমের জন্য বেছে নেবেন: জিরো-কস্ট অ্যাবস্ট্রাকশন, গার্বেজ কালেক্টর ছাড়া মেমরি সেফটি, এবং 25 সেকেন্ডের কম সময়ে 30,000+ ফাইল কোডবেস প্রসেস করার ক্ষমতা।</p>

<p>আমাদের গ্রাফে প্রতিটি রিজলভড এজ টাইপড মেটাডেটা বহন করে:</p>

<table>
<thead>
<tr><th>ফিল্ড</th><th>উদ্দেশ্য</th></tr>
</thead>
<tbody>
<tr><td><code>ReferenceKind</code></td><td>কী ধরনের রেফারেন্স (কল, ইমপোর্ট, extends, implements)</td></tr>
<tr><td><code>RelationKind</code></td><td>সিমান্টিক সম্পর্ক (ডিপেন্ডেন্সি, ইনহেরিট্যান্স, ইভেন্ট)</td></tr>
<tr><td><code>GraphLayer</code></td><td>স্ট্রাকচারাল, রানটাইম, ফ্রেমওয়ার্ক, বা পলিসি-ওভারলে</td></tr>
<tr><td><code>EdgeStrength</code></td><td>কাপলিং কতটা শক্তিশালী</td></tr>
<tr><td><code>EdgeOrigin</code></td><td>এজটি কোথায় আবিষ্কৃত হয়েছে (পার্সার, রিজলভার, plugin)</td></tr>
<tr><td><code>ResolutionTier</code></td><td>রেজোলিউশনে কতটা আত্মবিশ্বাস</td></tr>
</tbody>
</table>

<p>এর মানে প্রতিটি এজ শুধু "A B-র উপর নির্ভর করে" নয় — এটি "A B-র উপর নির্ভর করে <em>এই সম্পর্কের মাধ্যমে, এই লেয়ারে, এই আত্মবিশ্বাসে, এই কারণে</em>।" এই পার্থক্য ব্যাখ্যাযোগ্যতার জন্য এবং আমরা যে ডকট্রিন-ভিত্তিক আর্কিটেকচারাল বিচার তৈরি করছি তার জন্য গুরুত্বপূর্ণ।</p>

<h2 id="layered-meaning">লেয়ারড মিনিং</h2>

<p>আমরা উন্নয়নের প্রথম দিকেই ফ্ল্যাট এজ সেট থেকে সরে এসেছি। বর্তমান মডেল চারটি লেয়ার আলাদা করে:</p>

<ol>
<li><strong>স্ট্রাকচারাল এজ</strong> — সরাসরি ইমপোর্ট, ক্লাস রেফারেন্স, টাইপ অ্যানোটেশন</li>
<li><strong>রানটাইম এজ</strong> — কিউ ডিসপ্যাচ, ইভেন্ট এমিশন, ডায়নামিক রেজোলিউশন</li>
<li><strong>ফ্রেমওয়ার্ক এজ</strong> — IoC কন্টেইনার বাইন্ডিং, WordPress হুক, Laravel সার্ভিস প্রোভাইডার</li>
<li><strong>পলিসি-ওভারলে এজ</strong> — গৃহীত কোডবেস কনভেনশনের জন্য কনফিগারেশন নিয়ম দ্বারা যোগ করা এজ</li>
</ol>

<p>এই লেয়ারিং আমাদের বিভিন্ন গ্রাফ ভিউয়ের বিপরীতে মৌলিকভাবে ভিন্ন প্রশ্ন জিজ্ঞাসা করতে দেয়। আমরা রানটাইম-সম্প্রসারিত সাইকেল থেকে স্ট্রাকচারাল সাইকেল আলাদাভাবে শনাক্ত করতে পারি। আমরা ফ্রেমওয়ার্ক আর্টিফ্যাক্ট চিহ্নিত করতে পারি প্রকৃত কাপলিংয়ের সাথে বিভ্রান্ত না হয়ে। এবং আমরা ব্যবহারকারীদের পলিসি নিয়মের মাধ্যমে কোন প্যাটার্নগুলো ইচ্ছাকৃত তা ঘোষণা করতে দিতে পারি, কোর গ্রাফ পরিবর্তন না করে।</p>

<h2 id="plugin-expanded-framework-behavior">Plugin-সম্প্রসারিত ফ্রেমওয়ার্ক আচরণ</h2>

<p>আমাদের সবচেয়ে গুরুত্বপূর্ণ আর্কিটেকচারাল সিদ্ধান্তগুলোর একটি হলো ফ্রেমওয়ার্ক জ্ঞান কোর ল্যাঙ্গুয়েজ পার্সারে থাকে না। বরং, এটি <strong>plugin</strong>-এ থাকে:</p>

<ul>
<li><strong>কিউ plugin</strong> জব ডিসপ্যাচকে রানটাইম এজে সম্প্রসারিত করে</li>
<li><strong>কন্টেইনার plugin</strong> IoC বাইন্ডিংকে ফ্রেমওয়ার্ক এজে রিজলভ করে</li>
<li><strong>WordPress plugin</strong> <code>add_action</code> / <code>do_action</code>-কে পাবলিশ/সাবস্ক্রাইব এজে ম্যাপ করে</li>
</ul>

<p>নীতিটি সরল:</p>

<ul>
<li>ভাষার সত্য কোরে থাকে</li>
<li>ফ্রেমওয়ার্কের সত্য plugin-এ থাকে</li>
<li>রিপোজিটরি-নির্দিষ্ট গৃহীত আচরণ পলিসি নিয়মে থাকে</li>
</ul>

<p>এই বিভাজন ছাড়া, পণ্যটি রিপোজিটরি-নির্দিষ্ট হ্যাকে ভেঙে পড়তো। প্রতিটি WordPress ইনস্টলেশনে ভিন্ন হার্ডকোডেড প্যাটার্ন প্রয়োজন হতো। প্রতিটি Laravel সংস্করণ গ্রাফ ভেঙে দিতো। ফ্রেমওয়ার্ক জ্ঞান plugin-এ রেখে, আমরা কোর অ্যানালিসিস ইঞ্জিন থেকে স্বাধীনভাবে ফ্রেমওয়ার্ক সাপোর্ট বিকশিত করতে পারি।</p>

<h2 id="two-views-one-truth">দুটি ভিউ, একটি সত্য</h2>

<p>আমাদের সর্বশেষ ইটারেশনে সবচেয়ে গুরুত্বপূর্ণ সংশোধন ছিল <strong>ক্যানোনিক্যাল গ্রাফ</strong> থেকে <strong>ডিপেন্ডেন্সি ভিউ</strong> আলাদা করা।</p>

<p>আমাদের প্রাথমিক গ্রাফ এক্সপোর্ট অত্যন্ত নয়েজি ছিল। এতে প্রতিটি ফাইলের জন্য সিনথেটিক MODULE নোড, প্রতিটি সিম্বলের জন্য CONTAINS এজ, এবং পৃথকভাবে গণনা করা পুনরাবৃত্ত কল-সাইট এজ অন্তর্ভুক্ত ছিল। এটি গ্রাফটিকে চিত্তাকর্ষকভাবে বড় দেখাতো, কিন্তু এর অনেকটাই ছিল রিপ্রেজেন্টেশনাল ওভারহেড, আর্কিটেকচারাল মূল্য নয়।</p>

<p>আমরা এখন একই সত্যের উৎস থেকে দুটি ভিউ বজায় রাখি:</p>

<h3 id="canonical-graph">ক্যানোনিক্যাল গ্রাফ (এভিডেন্স-অপটিমাইজড)</h3>
<p>ক্যানোনিক্যাল গ্রাফ সবকিছু ধরে রাখে: পুনরাবৃত্ত কল সাইট, বিস্তারিত রানটাইম এবং plugin এজ, সূক্ষ্ম সিমান্টিক তথ্য, এবং গভীর তদন্তের জন্য প্রয়োজনীয় সমস্ত প্রমাণ। এটিই আমাদের ডিটেক্টর এবং AI রিভিউ পর্যায় চালায়।</p>

<h3 id="dependency-view">ডিপেন্ডেন্সি ভিউ (কোয়েরি-অপটিমাইজড)</h3>
<p>ডিপেন্ডেন্সি ভিউ হলো একটি নরমালাইজড প্রজেকশন যা সিনথেটিক নোড বাদ দেয়, কন্টেইনমেন্ট এজ বাদ দেয়, মডিউল-টার্গেটেড এজকে ফাইল নোডে রিম্যাপ করে, এবং পুনরাবৃত্ত ডিপেন্ডেন্সিকে একটি <code>occurrenceCount</code> সহ একটি একক এজে কোল্যাপ্স করে। এটিই আমাদের রিপোর্টিং, MCP অ্যাক্সেস এবং আর্কিটেকচার এক্সপ্লোরেশন চালায়।</p>

<p>অন্যভাবে বলতে গেলে: ক্যানোনিক্যাল গ্রাফ সত্য এবং প্রমাণের জন্য অপটিমাইজ করে। ডিপেন্ডেন্সি ভিউ কম-নয়েজ আর্কিটেকচারাল ইন্টারপ্রিটেশনের জন্য অপটিমাইজ করে।</p>

<h2 id="wordpress-benchmark">WordPress বেঞ্চমার্ক: 22.78 সেকেন্ডে 32,862 নোড</h2>

<p>আমরা WordPress-এর বিপরীতে বেঞ্চমার্ক করি — বিশ্বের অন্যতম বৃহত্তম এবং সবচেয়ে জটিল PHP কোডবেস। নরমালাইজড ডিপেন্ডেন্সি ভিউ থেকে আমাদের বর্তমান সংখ্যা:</p>

<table>
<thead>
<tr><th>মেট্রিক</th><th>AigisCode</th></tr>
</thead>
<tbody>
<tr><td>ওয়াল ক্লক</td><td>22.78s</td></tr>
<tr><td>নোড</td><td>32,862</td></tr>
<tr><td>সম্পর্ক</td><td>95,878</td></tr>
</tbody>
</table>

<p>সম্পর্কের ব্রেকডাউন আমাদের গ্রাফের সমৃদ্ধি প্রকাশ করে:</p>

<ul>
<li><strong>CALL</strong>: 85,451 — ফাংশন এবং মেথড ইনভোকেশন</li>
<li><strong>EVENTPUBLISH</strong>: 3,662 — WordPress হুক অ্যাক্টিভেশন (<code>do_action</code>, <code>apply_filters</code>)</li>
<li><strong>OVERRIDES</strong>: 1,947 — ক্লাস হায়ারার্কিতে মেথড ওভাররাইড</li>
<li><strong>EVENTSUBSCRIBE</strong>: 1,868 — হুক রেজিস্ট্রেশন (<code>add_action</code>, <code>add_filter</code>)</li>
<li><strong>EXTENDS</strong>: 1,489 — ক্লাস ইনহেরিট্যান্স</li>
<li><strong>IMPORT</strong>: 764 — ফাইল-লেভেল ইমপোর্ট এবং ইনক্লুড</li>
<li><strong>TYPEUSE</strong>: 625 — টাইপ অ্যানোটেশন এবং হিন্ট</li>
<li><strong>IMPLEMENTS</strong>: 72 — ইন্টারফেস ইমপ্লিমেন্টেশন</li>
</ul>

<p>WordPress হুক এজ (EVENTPUBLISH + EVENTSUBSCRIBE) বিশেষভাবে তাৎপর্যপূর্ণ। এগুলো রানটাইম ওয়্যারিং উপস্থাপন করে যা ফ্ল্যাট স্ট্যাটিক অ্যানালিসিস টুল সম্পূর্ণ মিস করে। যখন WordPress <code>do_action('init')</code> কল করে, 47টি ভিন্ন plugin সাড়া দেয়। আমাদের গ্রাফ সেই 47টি সংযোগের সবকটি ক্যাপচার করে।</p>

<h2 id="optional-kuzu-read-model">ঐচ্ছিক Kuzu রিড মডেল</h2>

<p>কোয়েরি এবং এক্সপ্লোরেশনের জন্য, আমরা ঐচ্ছিকভাবে ডিপেন্ডেন্সি ভিউ <a href="https://kuzudb.com/">Kuzu</a>-তে এক্সপোর্ট করি, একটি এম্বেডেড গ্রাফ ডেটাবেস। এটি আমাদের দেয়:</p>

<ul>
<li>অ্যাড-হক গ্রাফ এক্সপ্লোরেশনের জন্য Cypher কোয়েরি সাপোর্ট</li>
<li>AI এজেন্টদের গ্রাফ কোয়েরি করার জন্য MCP সার্ভার অ্যাক্সেস</li>
<li>আর্কিটেকচার আবিষ্কারের জন্য দ্রুত প্যাটার্ন ম্যাচিং</li>
</ul>

<p>মূল আর্কিটেকচারাল পছন্দ হলো Kuzu একটি <em>রিড মডেল</em>, সত্যের উৎস নয়। অ্যানালিসিস লজিক স্টোরেজ মেকানিক্সের সাথে কাপলড হওয়া উচিত নয়। Rust গ্রাফ এবং JSON আর্টিফ্যাক্ট পোর্টেবল, ক্যানোনিক্যাল রিপ্রেজেন্টেশন থাকে। Kuzu স্টোরেজ ডিপেন্ডেন্সি তৈরি না করে কোয়েরি পাওয়ার যোগ করে।</p>

<h2 id="what-this-enables">এটি কী সক্ষম করে</h2>

<p>একটি টাইপড, লেয়ারড, এভিডেন্স-প্রিজার্ভিং গ্রাফের সাথে, আমরা এমন ডিটেক্টর এবং গভর্নেন্স সিস্টেম তৈরি করতে পারি যা আগে অসম্ভব ছিল:</p>

<ul>
<li><strong>স্ট্রাকচারাল সাইকেল ডিটেকশন</strong> যা ইভেন্ট বাসের মাধ্যমে ইচ্ছাকৃত রানটাইম সাইকেল উপেক্ষা করে</li>
<li><strong>ডেড কোড ডিটেকশন</strong> যা বোঝে ফ্রেমওয়ার্ক-রিজলভড ক্লাস সত্যিই ডেড নয়</li>
<li><strong>গড ক্লাস আইডেন্টিফিকেশন</strong> যা গ্রাফের বিভিন্ন লেয়ারে কাপলিং বিবেচনা করে</li>
<li><strong>আর্কিটেকচার সারফেস জেনারেশন</strong> যা ডেভেলপারদের দেখায় প্রকৃত চাপের পয়েন্ট কোথায়</li>
<li><strong>AI-চালিত রিভিউ</strong> যা শুধু ফাইল-লেভেল হিউরিস্টিক নয়, সম্পূর্ণ গ্রাফ কনটেক্সট দিয়ে ফলাফল শ্রেণিবদ্ধ করে</li>
</ul>

<p>এটি একটি নয়েজি কোড গ্রাফ এবং একটি কার্যকর গার্ডিয়ান সিস্টেমের মধ্যে পার্থক্য। AigisCode নোড এবং এজ গণনা করার চেষ্টা করছে না। এটি মানুষ এবং AI-কে বুঝতে সাহায্য করার চেষ্টা করছে কিভাবে একটি কোডবেস আসলে ওয়্যার্ড — কোথায় আর্কিটেকচার সুস্থ, কোথায় অবনতি হচ্ছে, এবং এ বিষয়ে কী করা উচিত।</p>

<h2 id="try-it-yourself">নিজে চেষ্টা করুন</h2>

<p>AigisCode ওপেন সোর্স এবং MIT-লাইসেন্সড। আপনি আজই আপনার নিজের কোডবেসে এটি চালাতে পারেন:</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
aigiscode analyze /path/to/your/project
</code></pre>

<p>অ্যানালিসিস <code>.aigiscode/</code>-এ স্ট্রাকচার্ড JSON আর্টিফ্যাক্ট তৈরি করে যা যেকোনো AI এজেন্ট বা CI pipeline ব্যবহার করতে পারে। আপনার গ্রাফ কেমন দেখায় তা শুনতে আমরা আগ্রহী।</p>
`,
      zh: `
<p>大多数静态分析工具将你的代码库视为一堆文件。它们孤立地扫描每个文件，标记样式违规，然后继续。但真正的软件架构存在于<strong>文件之间的关系</strong>中——import、调用、继承链、事件订阅以及将一切连接在一起的运行时调度模式。</p>

<p>在 AigisCode，我们正在构建一些不同的东西：一个<strong>语义代码图</strong>，它不仅捕获什么依赖于什么，还捕获这些依赖<em>如何</em>存在、<em>为什么</em>存在以及存在于<em>哪个层级</em>。这是我们如何走到这一步的技术故事。</p>

<h2 id="why-flat-graphs-fail">为什么扁平代码图会失败</h2>

<p>扁平的依赖图只说"文件 A 依赖于文件 B。"这很有用，但有局限性。考虑一个 Laravel 应用：一个控制器调用一个服务，服务调度一个队列任务，任务通过 IoC 容器解析一个仓储。在扁平图中，你看到四个节点和三条边。但实际上，有三种不同<em>类型</em>的依赖在起作用：</p>

<ul>
<li><strong>结构性的</strong> — 导入服务类的 <code>use</code> 语句</li>
<li><strong>运行时的</strong> — 在运行时连接任务的队列调度</li>
<li><strong>框架的</strong> — IoC 管理的容器解析</li>
</ul>

<p>如果你将所有三种类型扁平化为相同的边类型，你就失去了对它们进行不同推理的能力。你无法区分结构性循环（总是有问题的）和通过事件总线的运行时循环（通常是有意的）。你无法判断一个"死"类是真的不可达，还是仅仅是通过你的工具不理解的框架约定来解析的。</p>

<p>这就是我们要解决的根本问题。</p>

<h2 id="the-canonical-rust-graph">规范的 Rust 图</h2>

<p>AigisCode 的真实来源是一个完全用原生 Rust 构建的语义图。我们选择 Rust 的原因与你选择它来构建任何性能关键系统的原因相同：零成本抽象、无垃圾回收器的内存安全，以及在 25 秒内处理 30,000+ 文件代码库的能力。</p>

<p>我们图中的每条已解析的边都携带类型化的元数据：</p>

<table>
<thead>
<tr><th>字段</th><th>用途</th></tr>
</thead>
<tbody>
<tr><td><code>ReferenceKind</code></td><td>引用类型（调用、导入、继承、实现）</td></tr>
<tr><td><code>RelationKind</code></td><td>语义关系（依赖、继承、事件）</td></tr>
<tr><td><code>GraphLayer</code></td><td>结构层、运行时层、框架层或策略覆盖层</td></tr>
<tr><td><code>EdgeStrength</code></td><td>耦合强度</td></tr>
<tr><td><code>EdgeOrigin</code></td><td>边的发现位置（解析器、解析器、插件）</td></tr>
<tr><td><code>ResolutionTier</code></td><td>解析的置信度</td></tr>
</tbody>
</table>

<p>这意味着每条边不仅仅是"A 依赖于 B"——而是"A <em>通过这种关系、在这个层级、以这种置信度、因为这个原因</em>依赖于 B。"这种区分对于可解释性以及我们正在构建的基于准则的架构判断至关重要。</p>

<h2 id="layered-meaning">分层语义</h2>

<p>我们在开发早期就放弃了扁平的边集。当前模型区分四个层级：</p>

<ol>
<li><strong>结构边</strong> — 直接导入、类引用、类型注解</li>
<li><strong>运行时边</strong> — 队列调度、事件发射、动态解析</li>
<li><strong>框架边</strong> — IoC 容器绑定、WordPress 钩子、Laravel 服务提供者</li>
<li><strong>策略覆盖边</strong> — 通过配置规则为已接受的代码库约定添加的边</li>
</ol>

<p>这种分层让我们能够针对不同的图视图提出根本不同的问题。我们可以将结构性循环与运行时扩展的循环分开检测。我们可以识别框架构件而不将它们与真正的耦合混淆。我们可以让用户通过策略规则声明哪些模式是有意的，而无需修改核心图。</p>

<h2 id="plugin-expanded-framework-behavior">插件扩展的框架行为</h2>

<p>我们最重要的架构决策之一是：框架知识不驻留在核心语言解析器中。相反，它驻留在<strong>插件</strong>中：</p>

<ul>
<li><strong>队列插件</strong>将任务调度扩展为运行时边</li>
<li><strong>容器插件</strong>将 IoC 绑定解析为框架边</li>
<li><strong>WordPress 插件</strong>将 <code>add_action</code> / <code>do_action</code> 映射为发布/订阅边</li>
</ul>

<p>原则很简单：</p>

<ul>
<li>语言真相属于核心</li>
<li>框架真相属于插件</li>
<li>特定仓库的已接受行为属于策略规则</li>
</ul>

<p>没有这种分离，产品将退化为特定仓库的黑客手段。每个 WordPress 安装都需要不同的硬编码模式。每个 Laravel 版本都会破坏图。通过将框架知识保留在插件中，我们可以独立于核心分析引擎发展框架支持。</p>

<h2 id="two-views-one-truth">两个视图，一个真相</h2>

<p>我们最新迭代中最重要的修正是将<strong>规范图</strong>与<strong>依赖视图</strong>分离。</p>

<p>我们最初的图导出太嘈杂了。它们为每个文件包含合成 MODULE 节点，为每个符号包含 CONTAINS 边，并且单独计算重复的调用点边。这使图看起来令人印象深刻地大，但其中大部分大小是表示开销，而不是架构价值。</p>

<p>我们现在从同一个真实来源维护两个视图：</p>

<h3 id="canonical-graph">规范图（证据优化）</h3>
<p>规范图保留一切：重复的调用点、详细的运行时和插件边、细粒度的语义信息，以及深入调查所需的所有证据。这就是驱动我们的检测器和 AI 审查阶段的东西。</p>

<h3 id="dependency-view">依赖视图（查询优化）</h3>
<p>依赖视图是一个规范化投影，它省略合成节点、省略包含边、将模块目标边重新映射到文件节点，并将重复的依赖折叠为带有 <code>occurrenceCount</code> 的单条边。这就是驱动我们的报告、MCP 访问和架构探索的东西。</p>

<p>换句话说：规范图优化真相和证据。依赖视图优化低噪声架构解释。</p>

<h2 id="wordpress-benchmark">WordPress 基准测试：22.78 秒处理 32,862 个节点</h2>

<p>我们以 WordPress——世界上最大、最复杂的 PHP 代码库之一——作为基准测试。以下是我们从规范化依赖视图获得的当前数据：</p>

<table>
<thead>
<tr><th>指标</th><th>AigisCode</th></tr>
</thead>
<tbody>
<tr><td>总耗时</td><td>22.78s</td></tr>
<tr><td>节点数</td><td>32,862</td></tr>
<tr><td>关系数</td><td>95,878</td></tr>
</tbody>
</table>

<p>关系分类揭示了我们图的丰富性：</p>

<ul>
<li><strong>CALL</strong>：85,451 — 函数和方法调用</li>
<li><strong>EVENTPUBLISH</strong>：3,662 — WordPress 钩子激活（<code>do_action</code>、<code>apply_filters</code>）</li>
<li><strong>OVERRIDES</strong>：1,947 — 类层次结构中的方法覆盖</li>
<li><strong>EVENTSUBSCRIBE</strong>：1,868 — 钩子注册（<code>add_action</code>、<code>add_filter</code>）</li>
<li><strong>EXTENDS</strong>：1,489 — 类继承</li>
<li><strong>IMPORT</strong>：764 — 文件级导入和包含</li>
<li><strong>TYPEUSE</strong>：625 — 类型注解和提示</li>
<li><strong>IMPLEMENTS</strong>：72 — 接口实现</li>
</ul>

<p>WordPress 钩子边（EVENTPUBLISH + EVENTSUBSCRIBE）特别重要。它们代表了扁平静态分析工具完全遗漏的运行时连接。当 WordPress 调用 <code>do_action('init')</code> 时，47 个不同的插件会响应。我们的图捕获了所有 47 个连接。</p>

<h2 id="optional-kuzu-read-model">可选的 Kuzu 读模型</h2>

<p>对于查询和探索，我们可选地将依赖视图导出到 <a href="https://kuzudb.com/">Kuzu</a>，一个嵌入式图数据库。这为我们提供了：</p>

<ul>
<li>Cypher 查询支持，用于即席图探索</li>
<li>MCP 服务器访问，供 AI 代理查询图</li>
<li>快速模式匹配，用于架构发现</li>
</ul>

<p>关键的架构选择是 Kuzu 是一个<em>读模型</em>，而不是真实来源。分析逻辑不应与存储机制耦合。Rust 图和 JSON 构件仍然是可移植的、规范的表示。Kuzu 在不创建存储依赖的情况下增加了查询能力。</p>

<h2 id="what-this-enables">这启用了什么</h2>

<p>有了类型化的、分层的、保留证据的图，我们可以构建以前不可能的检测器和治理系统：</p>

<ul>
<li><strong>结构性循环检测</strong>——忽略通过事件总线的有意运行时循环</li>
<li><strong>死代码检测</strong>——理解框架解析的类并非真正死亡</li>
<li><strong>上帝类识别</strong>——考虑不同图层级的耦合</li>
<li><strong>架构表面生成</strong>——向开发者展示真正的压力点在哪里</li>
<li><strong>AI 驱动的审查</strong>——使用完整的图上下文分类发现，而不仅仅是文件级启发式</li>
</ul>

<p>这就是嘈杂的代码图和有用的守护系统之间的区别。AigisCode 不是在试图计算节点和边。它试图帮助人类和 AI 理解代码库实际上是如何连接的——架构在哪里是健康的，在哪里正在退化，以及该怎么做。</p>

<h2 id="try-it-yourself">亲自试试</h2>

<p>AigisCode 是开源的，采用 MIT 许可证。你今天就可以在自己的代码库上运行它：</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
aigiscode analyze /path/to/your/project
</code></pre>

<p>分析会在 <code>.aigiscode/</code> 生成结构化的 JSON 构件，任何 AI 代理或 CI 流水线都可以使用。我们很想知道你的图是什么样子。</p>
`,
      hi: `
<p>अधिकांश स्टैटिक एनालिसिस टूल आपके कोडबेस को फ़ाइलों के एक ढेर के रूप में मानते हैं। वे प्रत्येक फ़ाइल को अलग-अलग स्कैन करते हैं, स्टाइल उल्लंघन फ़्लैग करते हैं, और आगे बढ़ जाते हैं। लेकिन वास्तविक सॉफ़्टवेयर आर्किटेक्चर <strong>फ़ाइलों के बीच संबंधों</strong> में रहता है — इम्पोर्ट, कॉल, इनहेरिटेंस चेन, इवेंट सब्सक्रिप्शन और रनटाइम डिस्पैच पैटर्न जो सब कुछ एक साथ जोड़ते हैं।</p>

<p>AigisCode में, हम कुछ अलग बना रहे हैं: एक <strong>सिमेंटिक कोड ग्राफ़</strong> जो न केवल यह कैप्चर करता है कि क्या किस पर निर्भर है, बल्कि वे निर्भरताएँ <em>कैसे</em>, <em>क्यों</em> और <em>किस लेयर पर</em> मौजूद हैं। यह इसकी तकनीकी कहानी है कि हम यहाँ कैसे पहुँचे।</p>

<h2 id="why-flat-graphs-fail">फ्लैट कोड ग्राफ़ क्यों विफल होते हैं</h2>

<p>एक फ्लैट डिपेंडेंसी ग्राफ़ कहता है "फ़ाइल A, फ़ाइल B पर निर्भर है।" यह उपयोगी है, लेकिन सीमित है। एक Laravel एप्लिकेशन पर विचार करें जहाँ एक कंट्रोलर एक सर्विस को कॉल करता है, जो एक कतारबद्ध जॉब डिस्पैच करता है, जो IoC कंटेनर के माध्यम से एक रिपॉज़िटरी को रिज़ॉल्व करता है। एक फ्लैट ग्राफ़ में, आप चार नोड और तीन एज देखते हैं। वास्तव में, तीन अलग-अलग <em>प्रकार</em> की निर्भरता काम कर रही है:</p>

<ul>
<li><strong>संरचनात्मक</strong> — सर्विस क्लास को इम्पोर्ट करने वाला <code>use</code> स्टेटमेंट</li>
<li><strong>रनटाइम</strong> — रनटाइम पर जॉब को वायर करने वाला क्यू डिस्पैच</li>
<li><strong>फ़्रेमवर्क</strong> — IoC द्वारा प्रबंधित कंटेनर रिज़ॉल्यूशन</li>
</ul>

<p>यदि आप तीनों को एक ही एज प्रकार में समतल कर देते हैं, तो आप उनके बारे में अलग-अलग तर्क करने की क्षमता खो देते हैं। आप एक संरचनात्मक चक्र (हमेशा समस्याग्रस्त) को इवेंट बस के माध्यम से रनटाइम चक्र (अक्सर जानबूझकर) से अलग नहीं कर सकते। आप यह नहीं बता सकते कि कोई "डेड" क्लास वास्तव में अगम्य है या बस किसी फ़्रेमवर्क कन्वेंशन के माध्यम से रिज़ॉल्व हो रही है जिसे आपका टूल नहीं समझता।</p>

<p>यह वह मूलभूत समस्या है जिसे हमने हल करने का लक्ष्य रखा।</p>

<h2 id="the-canonical-rust-graph">कैनोनिकल Rust ग्राफ़</h2>

<p>AigisCode में सत्य का स्रोत पूरी तरह से नेटिव Rust में बना एक सिमेंटिक ग्राफ़ है। हमने Rust को उन्हीं कारणों से चुना जिनके लिए आप इसे किसी भी प्रदर्शन-महत्वपूर्ण सिस्टम के लिए चुनेंगे: ज़ीरो-कॉस्ट एब्स्ट्रैक्शन, गार्बेज कलेक्टर के बिना मेमोरी सेफ्टी, और 25 सेकंड से कम में 30,000+ फ़ाइल कोडबेस प्रोसेस करने की क्षमता।</p>

<p>हमारे ग्राफ़ में हर रिज़ॉल्व्ड एज टाइप्ड मेटाडेटा ले जाता है:</p>

<table>
<thead>
<tr><th>फ़ील्ड</th><th>उद्देश्य</th></tr>
</thead>
<tbody>
<tr><td><code>ReferenceKind</code></td><td>संदर्भ का प्रकार (कॉल, इम्पोर्ट, एक्सटेंड्स, इम्प्लीमेंट्स)</td></tr>
<tr><td><code>RelationKind</code></td><td>सिमेंटिक संबंध (डिपेंडेंसी, इनहेरिटेंस, इवेंट)</td></tr>
<tr><td><code>GraphLayer</code></td><td>संरचनात्मक, रनटाइम, फ़्रेमवर्क, या पॉलिसी-ओवरले</td></tr>
<tr><td><code>EdgeStrength</code></td><td>कपलिंग कितनी मज़बूत है</td></tr>
<tr><td><code>EdgeOrigin</code></td><td>एज कहाँ खोजी गई (पार्सर, रिज़ॉल्वर, प्लगइन)</td></tr>
<tr><td><code>ResolutionTier</code></td><td>रिज़ॉल्यूशन कितना विश्वसनीय है</td></tr>
</tbody>
</table>

<p>इसका मतलब है कि हर एज सिर्फ "A, B पर निर्भर है" नहीं है — यह है "A, B पर <em>इस संबंध के माध्यम से, इस लेयर पर, इस विश्वसनीयता के साथ, इस कारण से</em> निर्भर है।" यह भेद व्याख्यात्मकता और उस सिद्धांत-आधारित आर्किटेक्चरल निर्णय के लिए महत्वपूर्ण है जिसे हम बना रहे हैं।</p>

<h2 id="layered-meaning">लेयर्ड मीनिंग</h2>

<p>हमने विकास के शुरुआती दौर में ही फ्लैट एज सेट से दूरी बना ली। वर्तमान मॉडल चार लेयर में भेद करता है:</p>

<ol>
<li><strong>संरचनात्मक एज</strong> — प्रत्यक्ष इम्पोर्ट, क्लास संदर्भ, टाइप एनोटेशन</li>
<li><strong>रनटाइम एज</strong> — क्यू डिस्पैच, इवेंट एमिशन, डायनामिक रिज़ॉल्यूशन</li>
<li><strong>फ़्रेमवर्क एज</strong> — IoC कंटेनर बाइंडिंग, WordPress हुक, Laravel सर्विस प्रोवाइडर</li>
<li><strong>पॉलिसी-ओवरले एज</strong> — स्वीकृत कोडबेस कन्वेंशन के लिए कॉन्फ़िगरेशन नियमों द्वारा जोड़ी गई एज</li>
</ol>

<p>यह लेयरिंग हमें विभिन्न ग्राफ़ व्यू के विरुद्ध मौलिक रूप से अलग प्रश्न पूछने देती है। हम संरचनात्मक चक्रों को रनटाइम-विस्तारित चक्रों से अलग से पहचान सकते हैं। हम फ़्रेमवर्क आर्टिफ़ैक्ट को वास्तविक कपलिंग के साथ भ्रमित किए बिना पहचान सकते हैं। और हम उपयोगकर्ताओं को पॉलिसी नियमों के माध्यम से यह घोषित करने दे सकते हैं कि कौन से पैटर्न जानबूझकर हैं, बिना कोर ग्राफ़ को संशोधित किए।</p>

<h2 id="plugin-expanded-framework-behavior">प्लगइन-विस्तारित फ़्रेमवर्क व्यवहार</h2>

<p>हमारे सबसे महत्वपूर्ण आर्किटेक्चरल निर्णयों में से एक यह है कि फ़्रेमवर्क ज्ञान कोर लैंग्वेज पार्सर में नहीं रहता। इसके बजाय, यह <strong>प्लगइन</strong> में रहता है:</p>

<ul>
<li><strong>क्यू प्लगइन</strong> जॉब डिस्पैच को रनटाइम एज में विस्तारित करता है</li>
<li><strong>कंटेनर प्लगइन</strong> IoC बाइंडिंग को फ़्रेमवर्क एज में रिज़ॉल्व करता है</li>
<li><strong>WordPress प्लगइन</strong> <code>add_action</code> / <code>do_action</code> को पब्लिश/सब्सक्राइब एज में मैप करता है</li>
</ul>

<p>सिद्धांत सरल है:</p>

<ul>
<li>भाषा का सत्य कोर में रहता है</li>
<li>फ़्रेमवर्क का सत्य प्लगइन में रहता है</li>
<li>रिपॉज़िटरी-विशिष्ट स्वीकृत व्यवहार पॉलिसी नियमों में रहता है</li>
</ul>

<p>इस पृथक्करण के बिना, उत्पाद रिपॉज़िटरी-विशिष्ट हैक में बदल जाएगा। हर WordPress इंस्टॉलेशन को अलग-अलग हार्डकोडेड पैटर्न की आवश्यकता होगी। हर Laravel संस्करण ग्राफ़ को तोड़ देगा। फ़्रेमवर्क ज्ञान को प्लगइन में रखकर, हम कोर एनालिसिस इंजन से स्वतंत्र रूप से फ़्रेमवर्क सपोर्ट विकसित कर सकते हैं।</p>

<h2 id="two-views-one-truth">दो व्यू, एक सत्य</h2>

<p>हमारी नवीनतम पुनरावृत्ति में सबसे महत्वपूर्ण सुधार <strong>कैनोनिकल ग्राफ़</strong> को <strong>डिपेंडेंसी व्यू</strong> से अलग करना था।</p>

<p>हमारे शुरुआती ग्राफ़ एक्सपोर्ट बहुत शोरगुल वाले थे। उनमें हर फ़ाइल के लिए सिंथेटिक MODULE नोड, हर सिंबल के लिए CONTAINS एज, और व्यक्तिगत रूप से गिने जाने वाले दोहराए गए कॉल-साइट एज शामिल थे। इससे ग्राफ़ प्रभावशाली रूप से बड़ा दिखता था, लेकिन उसका अधिकांश आकार प्रतिनिधित्व ओवरहेड था, आर्किटेक्चरल मूल्य नहीं।</p>

<p>अब हम एक ही सत्य स्रोत से दो व्यू बनाए रखते हैं:</p>

<h3 id="canonical-graph">कैनोनिकल ग्राफ़ (साक्ष्य-अनुकूलित)</h3>
<p>कैनोनिकल ग्राफ़ सब कुछ रखता है: दोहराए गए कॉल साइट, विस्तृत रनटाइम और प्लगइन एज, बारीक सिमेंटिक जानकारी, और गहन जाँच के लिए आवश्यक सभी साक्ष्य। यही हमारे डिटेक्टर और AI समीक्षा चरण को संचालित करता है।</p>

<h3 id="dependency-view">डिपेंडेंसी व्यू (क्वेरी-अनुकूलित)</h3>
<p>डिपेंडेंसी व्यू एक सामान्यीकृत प्रक्षेपण है जो सिंथेटिक नोड हटाता है, कंटेनमेंट एज हटाता है, मॉड्यूल-लक्षित एज को फ़ाइल नोड पर रीमैप करता है, और दोहराई गई निर्भरताओं को <code>occurrenceCount</code> वाली एकल एज में समेटता है। यही हमारी रिपोर्टिंग, MCP एक्सेस और आर्किटेक्चर अन्वेषण को संचालित करता है।</p>

<p>दूसरे शब्दों में: कैनोनिकल ग्राफ़ सत्य और साक्ष्य के लिए अनुकूलित है। डिपेंडेंसी व्यू कम-शोर आर्किटेक्चरल व्याख्या के लिए अनुकूलित है।</p>

<h2 id="wordpress-benchmark">WordPress बेंचमार्क: 22.78 सेकंड में 32,862 नोड</h2>

<p>हम WordPress — दुनिया के सबसे बड़े और सबसे जटिल PHP कोडबेस में से एक — के विरुद्ध बेंचमार्क करते हैं। सामान्यीकृत डिपेंडेंसी व्यू से हमारे वर्तमान आँकड़े:</p>

<table>
<thead>
<tr><th>मेट्रिक</th><th>AigisCode</th></tr>
</thead>
<tbody>
<tr><td>कुल समय</td><td>22.78s</td></tr>
<tr><td>नोड</td><td>32,862</td></tr>
<tr><td>रिलेशनशिप</td><td>95,878</td></tr>
</tbody>
</table>

<p>रिलेशनशिप का विश्लेषण हमारे ग्राफ़ की समृद्धि को प्रकट करता है:</p>

<ul>
<li><strong>CALL</strong>: 85,451 — फ़ंक्शन और मेथड इनवोकेशन</li>
<li><strong>EVENTPUBLISH</strong>: 3,662 — WordPress हुक एक्टिवेशन (<code>do_action</code>, <code>apply_filters</code>)</li>
<li><strong>OVERRIDES</strong>: 1,947 — क्लास हायरार्की में मेथड ओवरराइड</li>
<li><strong>EVENTSUBSCRIBE</strong>: 1,868 — हुक रजिस्ट्रेशन (<code>add_action</code>, <code>add_filter</code>)</li>
<li><strong>EXTENDS</strong>: 1,489 — क्लास इनहेरिटेंस</li>
<li><strong>IMPORT</strong>: 764 — फ़ाइल-स्तरीय इम्पोर्ट और इनक्लूड</li>
<li><strong>TYPEUSE</strong>: 625 — टाइप एनोटेशन और हिंट</li>
<li><strong>IMPLEMENTS</strong>: 72 — इंटरफ़ेस इम्प्लीमेंटेशन</li>
</ul>

<p>WordPress हुक एज (EVENTPUBLISH + EVENTSUBSCRIBE) विशेष रूप से महत्वपूर्ण हैं। ये रनटाइम वायरिंग का प्रतिनिधित्व करते हैं जिसे फ्लैट स्टैटिक एनालिसिस टूल पूरी तरह से चूक जाते हैं। जब WordPress <code>do_action('init')</code> कॉल करता है, तो 47 अलग-अलग प्लगइन प्रतिक्रिया करते हैं। हमारा ग्राफ़ उन सभी 47 कनेक्शन को कैप्चर करता है।</p>

<h2 id="optional-kuzu-read-model">वैकल्पिक Kuzu रीड मॉडल</h2>

<p>क्वेरी और अन्वेषण के लिए, हम वैकल्पिक रूप से डिपेंडेंसी व्यू को <a href="https://kuzudb.com/">Kuzu</a>, एक एम्बेडेड ग्राफ़ डेटाबेस में एक्सपोर्ट करते हैं। यह हमें प्रदान करता है:</p>

<ul>
<li>एड-हॉक ग्राफ़ अन्वेषण के लिए Cypher क्वेरी सपोर्ट</li>
<li>AI एजेंट को ग्राफ़ क्वेरी करने के लिए MCP सर्वर एक्सेस</li>
<li>आर्किटेक्चर डिस्कवरी के लिए तेज़ पैटर्न मैचिंग</li>
</ul>

<p>मुख्य आर्किटेक्चरल चयन यह है कि Kuzu एक <em>रीड मॉडल</em> है, सत्य का स्रोत नहीं। एनालिसिस लॉजिक को स्टोरेज मैकेनिक्स से युग्मित नहीं होना चाहिए। Rust ग्राफ़ और JSON आर्टिफ़ैक्ट पोर्टेबल, कैनोनिकल प्रतिनिधित्व बने रहते हैं। Kuzu स्टोरेज निर्भरता बनाए बिना क्वेरी शक्ति जोड़ता है।</p>

<h2 id="what-this-enables">यह क्या संभव बनाता है</h2>

<p>एक टाइप्ड, लेयर्ड, साक्ष्य-संरक्षित ग्राफ़ के साथ, हम ऐसे डिटेक्टर और गवर्नेंस सिस्टम बना सकते हैं जो पहले असंभव थे:</p>

<ul>
<li><strong>संरचनात्मक चक्र डिटेक्शन</strong> — इवेंट बस के माध्यम से जानबूझकर रनटाइम चक्रों को अनदेखा करता है</li>
<li><strong>डेड कोड डिटेक्शन</strong> — समझता है कि फ़्रेमवर्क-रिज़ॉल्व्ड क्लास वास्तव में डेड नहीं हैं</li>
<li><strong>गॉड क्लास पहचान</strong> — विभिन्न ग्राफ़ लेयर पर कपलिंग को ध्यान में रखता है</li>
<li><strong>आर्किटेक्चर सरफ़ेस जनरेशन</strong> — डेवलपर को दिखाता है कि वास्तविक प्रेशर पॉइंट कहाँ हैं</li>
<li><strong>AI-संचालित समीक्षा</strong> — पूर्ण ग्राफ़ संदर्भ के साथ निष्कर्षों को वर्गीकृत करता है, न कि केवल फ़ाइल-स्तरीय ह्यूरिस्टिक्स</li>
</ul>

<p>यही शोरगुल वाले कोड ग्राफ़ और एक उपयोगी गार्जियन सिस्टम के बीच का अंतर है। AigisCode नोड और एज गिनने की कोशिश नहीं कर रहा। यह मनुष्यों और AI को यह समझने में मदद करने की कोशिश कर रहा है कि कोडबेस वास्तव में कैसे वायर्ड है — आर्किटेक्चर कहाँ स्वस्थ है, कहाँ बिगड़ रहा है, और इसके बारे में क्या करना है।</p>

<h2 id="try-it-yourself">स्वयं आज़माएँ</h2>

<p>AigisCode ओपन सोर्स और MIT-लाइसेंस प्राप्त है। आप आज ही इसे अपने कोडबेस पर चला सकते हैं:</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
aigiscode analyze /path/to/your/project
</code></pre>

<p>एनालिसिस <code>.aigiscode/</code> पर संरचित JSON आर्टिफ़ैक्ट उत्पन्न करता है जिसे कोई भी AI एजेंट या CI पाइपलाइन उपयोग कर सकता है। हम जानना चाहेंगे कि आपका ग्राफ़ कैसा दिखता है।</p>
`,
      pt: `
<p>A maioria das ferramentas de analise estatica trata seu codebase como um conjunto de arquivos. Elas escaneiam cada arquivo isoladamente, sinalizam violacoes de estilo e seguem em frente. Mas a verdadeira arquitetura de software vive nos <strong>relacionamentos entre arquivos</strong> — os imports, as chamadas, as cadeias de heranca, as assinaturas de eventos e os padroes de despacho em tempo de execucao que conectam tudo.</p>

<p>No AigisCode, estamos construindo algo diferente: um <strong>grafo de codigo semantico</strong> que captura nao apenas o que depende do que, mas <em>como</em>, <em>por que</em> e <em>em que camada</em> essas dependencias existem. Esta e a historia tecnica de como chegamos aqui.</p>

<h2 id="why-flat-graphs-fail">Por que Grafos de Codigo Planos Falham</h2>

<p>Um grafo de dependencias plano diz "o arquivo A depende do arquivo B." Isso e util, mas limitado. Considere uma aplicacao Laravel onde um controller chama um servico, que despacha um job enfileirado, que resolve um repositorio atraves do container IoC. Em um grafo plano, voce ve quatro nos e tres arestas. Na realidade, tres <em>tipos</em> diferentes de dependencia estao em jogo:</p>

<ul>
<li><strong>Estrutural</strong> — a instrucao <code>use</code> importando a classe de servico</li>
<li><strong>Runtime</strong> — o despacho de fila que conecta o job em tempo de execucao</li>
<li><strong>Framework</strong> — a resolucao de container que o IoC gerencia</li>
</ul>

<p>Se voce achatar todos os tres no mesmo tipo de aresta, perde a capacidade de raciocinar sobre eles de forma diferente. Voce nao consegue distinguir um ciclo estrutural (sempre problematico) de um ciclo runtime atraves do barramento de eventos (frequentemente intencional). Voce nao consegue dizer se uma classe "morta" e verdadeiramente inalcancavel ou simplesmente resolvida atraves de uma convencao de framework que sua ferramenta nao entende.</p>

<p>Este e o problema fundamental que nos propusemos a resolver.</p>

<h2 id="the-canonical-rust-graph">O Grafo Canonico em Rust</h2>

<p>A fonte da verdade no AigisCode e um grafo semantico construido inteiramente em Rust nativo. Escolhemos Rust pelas mesmas razoes que voce o escolheria para qualquer sistema critico em performance: abstracoes de custo zero, seguranca de memoria sem coletor de lixo e a capacidade de processar codebases com mais de 30.000 arquivos em menos de 25 segundos.</p>

<p>Cada aresta resolvida em nosso grafo carrega metadados tipados:</p>

<table>
<thead>
<tr><th>Campo</th><th>Proposito</th></tr>
</thead>
<tbody>
<tr><td><code>ReferenceKind</code></td><td>Tipo de referencia (chamada, import, extends, implements)</td></tr>
<tr><td><code>RelationKind</code></td><td>O relacionamento semantico (dependencia, heranca, evento)</td></tr>
<tr><td><code>GraphLayer</code></td><td>Estrutural, runtime, framework ou policy-overlay</td></tr>
<tr><td><code>EdgeStrength</code></td><td>Quao forte e o acoplamento</td></tr>
<tr><td><code>EdgeOrigin</code></td><td>Onde a aresta foi descoberta (parser, resolver, plugin)</td></tr>
<tr><td><code>ResolutionTier</code></td><td>Quao confiavel e a resolucao</td></tr>
</tbody>
</table>

<p>Isso significa que cada aresta nao e apenas "A depende de B" — e "A depende de B <em>atraves desta relacao, nesta camada, com esta confianca, por esta razao</em>." Essa distincao e critica para explicabilidade e para o julgamento arquitetural baseado em doutrina que estamos construindo.</p>

<h2 id="layered-meaning">Significado em Camadas</h2>

<p>Abandonamos o conjunto de arestas plano no inicio do desenvolvimento. O modelo atual distingue quatro camadas:</p>

<ol>
<li><strong>Arestas estruturais</strong> — imports diretos, referencias de classe, anotacoes de tipo</li>
<li><strong>Arestas runtime</strong> — despacho de fila, emissao de eventos, resolucao dinamica</li>
<li><strong>Arestas de framework</strong> — bindings de container IoC, hooks WordPress, service providers Laravel</li>
<li><strong>Arestas de policy-overlay</strong> — arestas adicionadas por regras de configuracao para convencoes aceitas do codebase</li>
</ol>

<p>Essa estrutura em camadas nos permite fazer perguntas fundamentalmente diferentes contra diferentes visoes do grafo. Podemos detectar ciclos estruturais separadamente de ciclos expandidos em runtime. Podemos identificar artefatos de framework sem confundi-los com acoplamento real. E podemos permitir que usuarios declarem quais padroes sao intencionais atraves de regras de politica, sem modificar o grafo central.</p>

<h2 id="plugin-expanded-framework-behavior">Comportamento de Framework Expandido por Plugins</h2>

<p>Uma de nossas decisoes arquiteturais mais importantes e que o conhecimento de framework nao reside nos parsers de linguagem centrais. Em vez disso, reside em <strong>plugins</strong>:</p>

<ul>
<li>O <strong>plugin de fila</strong> expande o despacho de jobs em arestas runtime</li>
<li>O <strong>plugin de container</strong> resolve bindings IoC em arestas de framework</li>
<li>O <strong>plugin WordPress</strong> mapeia <code>add_action</code> / <code>do_action</code> em arestas publish/subscribe</li>
</ul>

<p>O principio e simples:</p>

<ul>
<li>A verdade da linguagem pertence ao nucleo</li>
<li>A verdade do framework pertence aos plugins</li>
<li>O comportamento aceito especifico do repositorio pertence as regras de politica</li>
</ul>

<p>Sem essa separacao, o produto desmoronaria em hacks especificos de repositorio. Cada instalacao WordPress exigiria padroes hardcoded diferentes. Cada versao do Laravel quebraria o grafo. Mantendo o conhecimento de framework em plugins, podemos evoluir o suporte a frameworks independentemente do motor de analise central.</p>

<h2 id="two-views-one-truth">Duas Visoes, Uma Verdade</h2>

<p>A correcao mais importante em nossa ultima iteracao foi separar o <strong>grafo canonico</strong> da <strong>visao de dependencias</strong>.</p>

<p>Nossas exportacoes de grafo iniciais eram muito ruidosas. Incluiam nos MODULE sinteticos para cada arquivo, arestas CONTAINS para cada simbolo e arestas de call-site repetidas contadas individualmente. Isso fazia o grafo parecer impressionantemente grande, mas grande parte desse tamanho era overhead representacional, nao valor arquitetural.</p>

<p>Agora mantemos duas visoes a partir da mesma fonte de verdade:</p>

<h3 id="canonical-graph">Grafo Canonico (Otimizado para Evidencias)</h3>
<p>O grafo canonico retem tudo: call sites repetidos, arestas runtime e de plugin detalhadas, informacao semantica granular e todas as evidencias necessarias para investigacao profunda. Isso e o que alimenta nossos detectores e o estagio de revisao por IA.</p>

<h3 id="dependency-view">Visao de Dependencias (Otimizada para Consultas)</h3>
<p>A visao de dependencias e uma projecao normalizada que omite nos sinteticos, omite arestas de contencao, remapeia arestas direcionadas a modulos para nos de arquivo e consolida dependencias repetidas em uma unica aresta com um <code>occurrenceCount</code>. Isso e o que alimenta nossos relatorios, acesso MCP e exploracao de arquitetura.</p>

<p>Em outras palavras: o grafo canonico otimiza para verdade e evidencia. A visao de dependencias otimiza para interpretacao arquitetural com baixo ruido.</p>

<h2 id="wordpress-benchmark">Benchmark WordPress: 32.862 Nos em 22,78 Segundos</h2>

<p>Fazemos benchmark contra o WordPress — um dos maiores e mais complexos codebases PHP do mundo. Aqui estao nossos numeros atuais da visao de dependencias normalizada:</p>

<table>
<thead>
<tr><th>Metrica</th><th>AigisCode</th></tr>
</thead>
<tbody>
<tr><td>Tempo total</td><td>22,78s</td></tr>
<tr><td>Nos</td><td>32.862</td></tr>
<tr><td>Relacionamentos</td><td>95.878</td></tr>
</tbody>
</table>

<p>A distribuicao de relacionamentos revela a riqueza do nosso grafo:</p>

<ul>
<li><strong>CALL</strong>: 85.451 — invocacoes de funcoes e metodos</li>
<li><strong>EVENTPUBLISH</strong>: 3.662 — ativacoes de hooks WordPress (<code>do_action</code>, <code>apply_filters</code>)</li>
<li><strong>OVERRIDES</strong>: 1.947 — overrides de metodos em hierarquias de classes</li>
<li><strong>EVENTSUBSCRIBE</strong>: 1.868 — registros de hooks (<code>add_action</code>, <code>add_filter</code>)</li>
<li><strong>EXTENDS</strong>: 1.489 — heranca de classes</li>
<li><strong>IMPORT</strong>: 764 — imports e includes em nivel de arquivo</li>
<li><strong>TYPEUSE</strong>: 625 — anotacoes e dicas de tipo</li>
<li><strong>IMPLEMENTS</strong>: 72 — implementacoes de interface</li>
</ul>

<p>As arestas de hooks WordPress (EVENTPUBLISH + EVENTSUBSCRIBE) sao particularmente significativas. Representam a conexao runtime que ferramentas de analise estatica plana perdem completamente. Quando o WordPress chama <code>do_action('init')</code>, 47 plugins diferentes respondem. Nosso grafo captura todas as 47 conexoes.</p>

<h2 id="optional-kuzu-read-model">Modelo de Leitura Kuzu Opcional</h2>

<p>Para consultas e exploracao, opcionalmente exportamos a visao de dependencias para o <a href="https://kuzudb.com/">Kuzu</a>, um banco de dados de grafo embutido. Isso nos da:</p>

<ul>
<li>Suporte a consultas Cypher para exploracao ad-hoc do grafo</li>
<li>Acesso ao servidor MCP para agentes de IA consultarem o grafo</li>
<li>Pattern matching rapido para descoberta de arquitetura</li>
</ul>

<p>A escolha arquitetural chave e que o Kuzu e um <em>modelo de leitura</em>, nao a fonte da verdade. A logica de analise nao deve ser acoplada a mecanica de armazenamento. O grafo Rust e os artefatos JSON permanecem como a representacao portavel e canonica. O Kuzu adiciona poder de consulta sem criar dependencia de armazenamento.</p>

<h2 id="what-this-enables">O que Isso Possibilita</h2>

<p>Com um grafo tipado, em camadas e que preserva evidencias, podemos construir detectores e sistemas de governanca que antes eram impossiveis:</p>

<ul>
<li><strong>Deteccao de ciclos estruturais</strong> que ignora ciclos runtime intencionais atraves de barramentos de eventos</li>
<li><strong>Deteccao de codigo morto</strong> que entende que classes resolvidas por framework nao estao verdadeiramente mortas</li>
<li><strong>Identificacao de god class</strong> que considera o acoplamento em diferentes camadas do grafo</li>
<li><strong>Geracao de superficie de arquitetura</strong> que mostra aos desenvolvedores onde estao os verdadeiros pontos de pressao</li>
<li><strong>Revisao alimentada por IA</strong> que classifica achados com contexto completo do grafo, nao apenas heuristicas em nivel de arquivo</li>
</ul>

<p>Esta e a diferenca entre um grafo de codigo ruidoso e um sistema guardiao util. O AigisCode nao esta tentando contar nos e arestas. Esta tentando ajudar humanos e IA a entender como um codebase esta realmente conectado — onde a arquitetura esta saudavel, onde esta degradando e o que fazer a respeito.</p>

<h2 id="try-it-yourself">Experimente Voce Mesmo</h2>

<p>O AigisCode e open source e licenciado sob MIT. Voce pode executa-lo no seu proprio codebase hoje:</p>

<pre><code>curl -fsSL https://raw.githubusercontent.com/Draivix/aigiscode/main/install.sh | bash
aigiscode analyze /path/to/your/project
</code></pre>

<p>A analise produz artefatos JSON estruturados em <code>.aigiscode/</code> que qualquer agente de IA ou pipeline CI pode consumir. Adorariamos saber como e o seu grafo.</p>
`,
    },
  },

  /* ======================================================================== */
  /*  1. Why AI-Powered Code Analysis Matters in 2026                         */
  /* ======================================================================== */
  {
    slug: 'why-ai-code-analysis-matters-2026',
    date: '2026-02-24',
    readTime: 9,
    tags: ['AI', 'Code Quality', 'Trends'],
    image: '/blog-ai-code-analysis.jpg',
    author: { name: 'David Strejc', role: 'Creator of AigisCode' },
    relatedSlugs: [
      'static-analysis-vs-linters-2026',
      'ai-agents-code-quality-workflow',
    ],
    title: {
      en: 'Why AI-Powered Code Analysis Matters in 2026',
      cs: 'Proč na AI analýze kódu v roce 2026 záleží',
      fr: "Pourquoi l'analyse de code par IA compte en 2026",
      es: 'Por que el analisis de codigo con IA importa en 2026',
      zh: '2026 年 AI 代码分析为何重要',
      hi: '2026 में AI-संचालित कोड विश्लेषण क्यों मायने रखता है',
      pt: 'Por que a análise de código com IA importa em 2026',
      ar: 'لماذا يهم تحليل الشيفرة المدعوم بالذكاء الاصطناعي في 2026',
      pl: 'Dlaczego analiza kodu wspierana AI ma znaczenie w 2026 roku',
      bn: '২০২৬ সালে AI-চালিত কোড বিশ্লেষণ কেন গুরুত্বপূর্ণ',
    },
    description: {
      en: 'How AI is transforming static analysis beyond what traditional linters can achieve, and why architectural understanding is the next frontier for code quality tools.',
      cs: 'Jak AI mění statickou analýzu a proč je architektonické porozumění další hranicí pro nástroje kvality kódu.',
      fr: "Comment l'IA transforme l'analyse statique et pourquoi la comprehension architecturale est la prochaine frontiere.",
      es: 'Como la IA esta transformando el analisis estatico y por que la comprension arquitectonica es la proxima frontera.',
      zh: 'AI 如何改变静态分析，以及为什么架构理解是代码质量工具的下一个前沿。',
      hi: 'AI स्टैटिक विश्लेषण को कैसे बदल रहा है और आर्किटेक्चरल समझ अगली सीमा क्यों है।',
      pt: 'Como a IA está transformando a análise estática e por que a compreensão arquitetural é a próxima fronteira.',
      ar: 'كيف يحوّل الذكاء الاصطناعي التحليل الثابت بما يتجاوز ما يمكن لأدوات الفحص التقليدية تحقيقه ولماذا يعد الفهم المعماري الحدود التالية لأدوات جودة الشيفرة.',
      pl: 'Jak AI przekształca analizę statyczną poza możliwości tradycyjnych linterów i dlaczego zrozumienie architektury to kolejna granica dla narzędzi jakości kodu.',
      bn: 'AI কিভাবে প্রচলিত লিন্টারের সীমানা ছাড়িয়ে স্ট্যাটিক অ্যানালিসিস রূপান্তর করছে এবং আর্কিটেকচারাল বোঝাপড়া কেন কোড কোয়ালিটি টুলসের পরবর্তী সীমানা।',
    },
    metaDescription: {
      en: 'Discover why AI-powered code analysis is essential in 2026. Learn how tools like AigisCode go beyond linters to detect architectural issues, circular dependencies, and dead code across entire codebases.',
      cs: 'Zjistěte, proč je AI analýza kódu v roce 2026 nezbytná a jak nástroje jako AigisCode překonávají klasické lintery.',
      fr: "Decouvrez pourquoi l'analyse de code par IA est essentielle en 2026 et comment des outils comme AigisCode depassent les linters.",
      es: 'Descubra por que el analisis de codigo con IA es esencial en 2026 y como herramientas como AigisCode superan a los linters.',
      zh: '了解为什么 AI 代码分析在 2026 年至关重要，以及 AigisCode 等工具如何超越 linter。',
      hi: 'जानें कि 2026 में AI कोड विश्लेषण क्यों आवश्यक है और AigisCode जैसे उपकरण लिंटर्स से कैसे आगे जाते हैं।',
      pt: 'Descubra por que a análise de código com IA é essencial em 2026 e como ferramentas como o AigisCode superam os linters.',
      ar: 'اكتشف لماذا أصبح تحليل الشيفرة المدعوم بالذكاء الاصطناعي ضرورياً في عام 2026. تعرّف كيف تتجاوز أدوات مثل AigisCode أدوات الفحص لاكتشاف المشكلات المعمارية والتبعيات الدائرية والشيفرة الميتة عبر قواعد الشيفرة بأكملها.',
      pl: 'Odkryj, dlaczego analiza kodu wspierana AI jest niezbędna w 2026 roku. Dowiedz się, jak narzędzia takie jak AigisCode wychodzą poza lintery, wykrywając cykliczne zależności, martwy kod i problemy architektoniczne w całej bazie kodu.',
      bn: 'জানুন কেন ২০২৬ সালে AI-চালিত কোড বিশ্লেষণ অপরিহার্য। AigisCode-এর মতো টুলস কিভাবে লিন্টারের বাইরে গিয়ে আর্কিটেকচারাল সমস্যা, সার্কুলার ডিপেন্ডেন্সি এবং ডেড কোড শনাক্ত করে তা শিখুন।',
    },
    content: {
      en: `
<p>The year 2026 has brought a fundamental shift in how we think about code quality. We have moved past the era where running a linter was considered sufficient due diligence. Today, codebases span hundreds of thousands of files, microservices communicate through invisible contracts, and AI coding agents generate code at a pace human review alone cannot match. In this landscape, AI-powered code analysis is not a luxury. It is a necessity.</p>

<h2 id="the-limitations-of-traditional-linters">The Limitations of Traditional Linters</h2>

<p>Traditional linters like ESLint, Pylint, and PHPStan are excellent at what they do. They catch syntax errors, enforce style conventions, flag unused variables, and ensure consistent formatting. But they operate within a fundamental constraint: they analyze files in isolation.</p>

<p>Consider a large Django application with 400 Python modules. Pylint can tell you that a particular import is unused <em>within a single file</em>. It cannot tell you that module A depends on module B, which depends on module C, which depends back on module A, creating a circular dependency that makes the entire subsystem impossible to test in isolation. It cannot tell you that a utility class in <code>utils/helpers.py</code> is imported by 47 different files, making it a dangerous bottleneck where a single change cascades unpredictably. It cannot tell you that an entire directory of handler classes has been orphaned since the last refactor, sitting there accumulating dust and confusing every new developer who encounters it.</p>

<p>These are <strong>architectural issues</strong>. They exist in the relationships between files, not within them. And they are the issues that cost engineering teams the most time, cause the most production incidents, and create the most frustration during onboarding.</p>

<h2 id="what-ai-brings-to-static-analysis">What AI Brings to Static Analysis</h2>

<p>AI-powered static analysis operates at a different level of abstraction. Instead of examining individual files, it builds a <strong>dependency graph</strong> of your entire codebase. It understands which modules import which, how symbols flow between files, and where the structural pressure points are.</p>

<p>This is where tools like AigisCode differ from traditional approaches. AigisCode runs a six-stage pipeline: Index, Graph, Detect, Rules, AI Review, and Report. The first three stages are purely deterministic. Tree-sitter parses source files across PHP, Python, TypeScript, JavaScript, and Vue. petgraph constructs a full dependency graph. Detectors identify candidates for circular dependencies, dead code, hardwired values, and architectural violations.</p>

<p>The AI enters at stage five. After the deterministic detectors have produced candidates with confidence levels, the AI review stage classifies findings as true positives, false positives, or needs-context. This hybrid approach is critical. The deterministic stages ensure reproducibility and explainability. The AI stage handles the nuance that pure heuristics cannot capture.</p>

<h2 id="the-rise-of-ai-coding-agents">The Rise of AI Coding Agents</h2>

<p>Perhaps the most significant trend of 2025 and 2026 has been the proliferation of AI coding agents. Tools like Claude Code, GitHub Copilot Workspace, and Codex agents can now autonomously implement features, fix bugs, and refactor code. But these agents need structured, machine-readable feedback about codebase health to operate effectively.</p>

<p>A linter's output is useful for an AI agent, but limited. An agent can fix an unused import warning. But can it understand that fixing a circular dependency between three modules requires restructuring the shared abstractions? Can it decide which of 23 dead code findings to address first based on the risk profile of the surrounding code?</p>

<p>This is where AI-powered analysis tools become the sensory system for AI agents. AigisCode outputs a structured JSON report at <code>.aigiscode/deterministic-analysis.json</code> that an agent can parse directly. The report includes severity levels, confidence scores, file paths, and explanations. An agent can read this report, triage findings by confidence, and begin fixing the most impactful issues automatically.</p>

<h2 id="the-numbers-tell-the-story">The Numbers Tell the Story</h2>

<p>The impact of architectural issues is well-documented. A 2025 study by Stripe found that developers spend an estimated <strong>42% of their time</strong> dealing with technical debt and maintenance, up from 33% in 2018. Circular dependencies are a leading cause of "dependency hell" in large monoliths, and a 2024 analysis of 10,000 open-source Python projects found that <strong>34% contained at least one strong circular dependency</strong> that affected test isolation.</p>

<p>Dead code is equally insidious. Research from the University of Zurich in 2025 estimated that the average enterprise codebase contains <strong>12-18% dead code</strong> by volume. This dead code increases build times, expands the security attack surface, confuses developers reading the code, and inflates bundle sizes for frontend applications.</p>

<p>Traditional linters catch none of this. A file containing dead code is syntactically valid. A circular dependency involves files that are individually correct. The problems only become visible when you look at the codebase as a whole.</p>

<h2 id="how-aigiscode-fits-the-ecosystem">How AigisCode Fits the Ecosystem</h2>

<p>AigisCode does not replace your linter. It complements it. Think of linters as your spell-checker and AigisCode as your structural editor. ESLint ensures your JavaScript follows consistent patterns. AigisCode ensures your modules do not form dependency cycles that make your application impossible to deploy incrementally.</p>

<p>The tool is open source under the MIT license and installable from GitHub or via cargo. A single command, <code>aigiscode analyze /path/to/project</code>, runs the full six-stage pipeline. The output is a machine-readable JSON report that both humans and AI agents can consume. Policy files allow teams to customize detection behavior per project, suppressing false positives and encoding project-specific knowledge.</p>

<p>Built-in plugin profiles for Django, Laravel, and WordPress handle framework-specific patterns out of the box. For example, the Django plugin recognizes that model classes referenced in migrations are not dead code, even if no Python file imports them directly. The Laravel plugin understands service provider bindings and facade accessors. These kinds of framework-aware adjustments eliminate entire categories of false positives without requiring manual configuration.</p>

<h2 id="looking-ahead">Looking Ahead</h2>

<p>The trajectory is clear. As codebases grow larger and AI agents take on more development work, the need for structural analysis that goes beyond file-level linting will only increase. We are moving toward a world where every pull request is evaluated not just for code style and test coverage, but for its impact on the dependency graph, its contribution to technical debt, and its alignment with the intended architecture.</p>

<p>AI-powered code analysis is the bridge between today's linters and tomorrow's fully automated code quality systems. The teams that adopt it now will have cleaner architectures, faster onboarding, and fewer production surprises. The tools exist. The patterns are proven. The question is no longer whether AI code analysis matters. It is whether your team can afford to work without it.</p>
`,
      cs: `
<p>Rok 2026 přinesl zásadní posun v tom, jak přemýšlíme o kvalitě kódu. Posunuli jsme se za éru, kdy bylo spuštění linteru považováno za dostatečnou péči. Dnes codebase zahrnují stovky tisíc souborů, mikroslužby komunikují přes neviditelné kontrakty a AI agenti generují kód tempem, které samotná lidská kontrola nedokáže zvládnout. V tomto prostředí není AI analýza kódu luxusem. Je nezbytností.</p>

<h2 id="the-limitations-of-traditional-linters">Omezení tradičních linterů</h2>

<p>Tradiční lintery jako ESLint, Pylint a PHPStan vynikají v tom, co dělají. Zachytí syntaktické chyby, vynucují stylové konvence, označují nepoužívané proměnné a zajišťují konzistentní formátování. Ale operují v rámci zásadního omezení: analyzují soubory izolovaně.</p>

<p>Představte si velkou Django aplikaci se 400 Python moduly. Pylint vám může říct, že konkrétní import je nepoužívaný <em>v rámci jednoho souboru</em>. Nemůže vám říct, že modul A závisí na modulu B, který závisí na modulu C, který závisí zpět na modulu A, čímž vzniká cyklická závislost, která znemožňuje testování celého subsystému izolovaně. Nemůže vám říct, že utilitní třída v <code>utils/helpers.py</code> je importována 47 různými soubory, což z ní dělá nebezpečné úzké hrdlo, kde jediná změna kaskádovitě působí nepředvídatelně. Nemůže vám říct, že celý adresář handler tříd osiřel od posledního refaktoringu a tam sedí, hromadí prach a mate každého nového vývojáře, který na něj narazí.</p>

<p>Toto jsou <strong>architektonické problémy</strong>. Existují ve vztazích mezi soubory, nikoli uvnitř nich. A jsou to problémy, které stojí inženýrské týmy nejvíce času, způsobují nejvíce produkčních incidentů a vytvářejí nejvíce frustrace při onboardingu.</p>

<h2 id="what-ai-brings-to-static-analysis">Co AI přináší statické analýze</h2>

<p>AI statická analýza operuje na jiné úrovni abstrakce. Místo zkoumání jednotlivých souborů buduje <strong>graf závislostí</strong> celého vašeho codebase. Rozumí tomu, které moduly importují které, jak symboly proudí mezi soubory a kde jsou strukturální tlakové body.</p>

<p>Zde se nástroje jako AigisCode liší od tradičních přístupů. AigisCode provádí šestistupňovou pipeline: Indexace, Graf, Detekce, Pravidla, AI Review a Report. První tři fáze jsou čistě deterministické. Tree-sitter parsuje zdrojové soubory napříč PHP, Pythonem, TypeScriptem, JavaScriptem a Vue. petgraph konstruuje kompletní graf závislostí. Detektory identifikují kandidáty na cyklické závislosti, mrtvý kód, natvrdo zapsané hodnoty a architektonická porušení.</p>

<p>AI vstupuje ve fázi pět. Poté, co deterministické detektory vygenerovaly kandidáty s úrovněmi spolehlivosti, fáze AI review klasifikuje nálezy jako true positive, false positive nebo vyžadující kontext. Tento hybridní přístup je klíčový. Deterministické fáze zajišťují reprodukovatelnost a vysvětlitelnost. AI fáze zvládá nuance, které čisté heuristiky nedokáží zachytit.</p>

<h2 id="the-rise-of-ai-coding-agents">Vzestup AI agentů pro kódování</h2>

<p>Pravděpodobně nejvýznamnějším trendem let 2025 a 2026 bylo rozšíření AI agentů pro kódování. Nástroje jako Claude Code, GitHub Copilot Workspace a Codex agenti nyní mohou autonomně implementovat funkce, opravovat chyby a refaktorovat kód. Ale tito agenti potřebují strukturovanou, strojově čitelnou zpětnou vazbu o zdraví codebase, aby mohli efektivně pracovat.</p>

<p>Výstup linteru je pro AI agenta užitečný, ale omezený. Agent dokáže opravit varování o nepoužitém importu. Ale dokáže pochopit, že oprava cyklické závislosti mezi třemi moduly vyžaduje restrukturalizaci sdílených abstrakcí? Dokáže rozhodnout, kterých z 23 nálezů mrtvého kódu se má věnovat nejdříve na základě rizikového profilu okolního kódu?</p>

<p>Zde se nástroje AI analýzy stávají senzorickým systémem pro AI agenty. AigisCode generuje strukturovaný JSON report v <code>.aigiscode/deterministic-analysis.json</code>, který agent může přímo parsovat. Report obsahuje úrovně závažnosti, skóre spolehlivosti, cesty k souborům a vysvětlení. Agent může tento report přečíst, roztřídit nálezy podle spolehlivosti a začít automaticky opravovat nejdůležitější problémy.</p>

<h2 id="the-numbers-tell-the-story">Čísla mluví za vše</h2>

<p>Dopad architektonických problémů je dobře zdokumentován. Studie Stripe z roku 2025 zjistila, že vývojáři tráví odhadem <strong>42 % svého času</strong> řešením technického dluhu a údržbou, oproti 33 % v roce 2018. Cyklické závislosti jsou hlavní příčinou „pekla závislostí" ve velkých monolitech a analýza 10 000 open-source Python projektů z roku 2024 zjistila, že <strong>34 % obsahovalo alespoň jednu silnou cyklickou závislost</strong>, která ovlivňovala izolaci testů.</p>

<p>Mrtvý kód je stejně zákeřný. Výzkum Univerzity v Curychu z roku 2025 odhadl, že průměrný podnikový codebase obsahuje <strong>12–18 % mrtvého kódu</strong> podle objemu. Tento mrtvý kód zvyšuje dobu sestavení, rozšiřuje plochu pro bezpečnostní útoky, mate vývojáře čtoucí kód a nafukuje velikost bundlů pro frontendové aplikace.</p>

<p>Tradiční lintery nic z toho nezachytí. Soubor obsahující mrtvý kód je syntakticky validní. Cyklická závislost zahrnuje soubory, které jsou jednotlivě správné. Problémy se stávají viditelnými teprve tehdy, když se podíváte na codebase jako celek.</p>

<h2 id="how-aigiscode-fits-the-ecosystem">Jak AigisCode zapadá do ekosystému</h2>

<p>AigisCode nenahrazuje váš linter. Doplňuje ho. Představte si lintery jako kontrolu pravopisu a AigisCode jako strukturálního editora. ESLint zajišťuje, že váš JavaScript dodržuje konzistentní vzory. AigisCode zajišťuje, že vaše moduly netvoří cykly závislostí, které znemožňují inkrementální nasazení aplikace.</p>

<p>Nástroj je open source pod licencí MIT. Jediný příkaz, <code>aigiscode analyze /path/to/project</code>, spustí kompletní šestistupňovou pipeline. Výstupem je strojově čitelný JSON report, který mohou konzumovat jak lidé, tak AI agenti. Policy soubory umožňují týmům přizpůsobit chování detekce pro každý projekt, potlačovat false positives a kódovat znalosti specifické pro projekt.</p>

<p>Vestavěné pluginové profily pro Django, Laravel a WordPress zvládají vzory specifické pro framework bez další konfigurace. Například Django plugin rozpozná, že třídy modelů odkazované v migracích nejsou mrtvý kód, i když je žádný Python soubor přímo neimportuje. Laravel plugin rozumí vazbám service providerů a přístupovým bodům fasád. Tyto druhy úprav zohledňujících framework eliminují celé kategorie false positives bez nutnosti manuální konfigurace.</p>

<h2 id="looking-ahead">Pohled vpřed</h2>

<p>Trajektorie je jasná. Jak codebase rostou a AI agenti přebírají více vývojové práce, potřeba strukturální analýzy překračující úroveň lintingu jednotlivých souborů bude jen narůstat. Směřujeme ke světu, kde každý pull request bude hodnocen nejen z hlediska stylu kódu a pokrytí testy, ale také z hlediska jeho dopadu na graf závislostí, jeho příspěvku k technickému dluhu a jeho souladu se zamýšlenou architekturou.</p>

<p>AI analýza kódu je mostem mezi dnešními lintery a zítřejšími plně automatizovanými systémy kvality kódu. Týmy, které ji přijmou nyní, budou mít čistší architektury, rychlejší onboarding a méně produkčních překvapení. Nástroje existují. Vzory jsou ověřené. Otázka už není, zda na AI analýze kódu záleží. Je to, zda si váš tým může dovolit pracovat bez ní.</p>
`,
      fr: `
<p>L'année 2026 a apporté un changement fondamental dans notre façon de concevoir la qualité du code. Nous avons dépassé l'époque où exécuter un linter était considéré comme une diligence suffisante. Aujourd'hui, les bases de code s'étendent sur des centaines de milliers de fichiers, les microservices communiquent via des contrats invisibles, et les agents de codage IA génèrent du code à un rythme que la revue humaine seule ne peut suivre. Dans ce paysage, l'analyse de code par IA n'est pas un luxe. C'est une nécessité.</p>

<h2 id="the-limitations-of-traditional-linters">Les limites des linters traditionnels</h2>

<p>Les linters traditionnels comme ESLint, Pylint et PHPStan excellent dans ce qu'ils font. Ils détectent les erreurs de syntaxe, imposent les conventions de style, signalent les variables inutilisées et assurent un formatage cohérent. Mais ils opèrent dans une contrainte fondamentale : ils analysent les fichiers de manière isolée.</p>

<p>Prenons une grande application Django avec 400 modules Python. Pylint peut vous dire qu'un import particulier est inutilisé <em>au sein d'un seul fichier</em>. Il ne peut pas vous dire que le module A dépend du module B, qui dépend du module C, qui dépend à son tour du module A, créant une dépendance circulaire qui rend l'ensemble du sous-système impossible à tester isolément. Il ne peut pas vous dire qu'une classe utilitaire dans <code>utils/helpers.py</code> est importée par 47 fichiers différents, en faisant un goulot d'étranglement dangereux où un seul changement se propage de manière imprévisible. Il ne peut pas vous dire qu'un répertoire entier de classes de gestionnaires est devenu orphelin depuis le dernier refactoring, restant là à accumuler la poussière et à dérouter chaque nouveau développeur qui le rencontre.</p>

<p>Ce sont des <strong>problèmes architecturaux</strong>. Ils existent dans les relations entre les fichiers, pas à l'intérieur de ceux-ci. Et ce sont les problèmes qui coûtent le plus de temps aux équipes d'ingénierie, causent le plus d'incidents en production et créent le plus de frustration lors de l'intégration des nouveaux arrivants.</p>

<h2 id="what-ai-brings-to-static-analysis">Ce que l'IA apporte à l'analyse statique</h2>

<p>L'analyse statique alimentée par l'IA opère à un niveau d'abstraction différent. Au lieu d'examiner des fichiers individuels, elle construit un <strong>graphe de dépendances</strong> de l'ensemble de votre base de code. Elle comprend quels modules importent lesquels, comment les symboles circulent entre les fichiers et où se trouvent les points de pression structurels.</p>

<p>C'est là que des outils comme AigisCode se distinguent des approches traditionnelles. AigisCode exécute un pipeline en six étapes : Index, Graph, Detect, Rules, AI Review et Report. Les trois premières étapes sont purement déterministes. Tree-sitter analyse les fichiers source en PHP, Python, TypeScript, JavaScript et Vue. petgraph construit un graphe de dépendances complet. Les détecteurs identifient les candidats pour les dépendances circulaires, le code mort, les valeurs codées en dur et les violations architecturales.</p>

<p>L'IA intervient à la cinquième étape. Après que les détecteurs déterministes ont produit des candidats avec des niveaux de confiance, l'étape de revue IA classe les résultats en vrais positifs, faux positifs ou nécessitant-un-contexte. Cette approche hybride est essentielle. Les étapes déterministes garantissent la reproductibilité et l'explicabilité. L'étape IA gère la nuance que les heuristiques pures ne peuvent capturer.</p>

<h2 id="the-rise-of-ai-coding-agents">L'essor des agents de codage IA</h2>

<p>La tendance la plus significative de 2025 et 2026 a sans doute été la prolifération des agents de codage IA. Des outils comme Claude Code, GitHub Copilot Workspace et les agents Codex peuvent désormais implémenter des fonctionnalités, corriger des bogues et refactoriser du code de manière autonome. Mais ces agents ont besoin de retours structurés et lisibles par machine sur la santé de la base de code pour fonctionner efficacement.</p>

<p>La sortie d'un linter est utile pour un agent IA, mais limitée. Un agent peut corriger un avertissement d'import inutilisé. Mais peut-il comprendre que la résolution d'une dépendance circulaire entre trois modules nécessite une restructuration des abstractions partagées ? Peut-il décider lequel des 23 résultats de code mort traiter en premier en fonction du profil de risque du code environnant ?</p>

<p>C'est là que les outils d'analyse alimentés par l'IA deviennent le système sensoriel des agents IA. AigisCode produit un rapport JSON structuré dans <code>.aigiscode/deterministic-analysis.json</code> qu'un agent peut analyser directement. Le rapport inclut les niveaux de sévérité, les scores de confiance, les chemins de fichiers et les explications. Un agent peut lire ce rapport, trier les résultats par confiance et commencer à corriger automatiquement les problèmes les plus impactants.</p>

<h2 id="the-numbers-tell-the-story">Les chiffres parlent d'eux-mêmes</h2>

<p>L'impact des problèmes architecturaux est bien documenté. Une étude de 2025 par Stripe a révélé que les développeurs consacrent environ <strong>42 % de leur temps</strong> à la dette technique et à la maintenance, contre 33 % en 2018. Les dépendances circulaires sont une cause majeure de l'« enfer des dépendances » dans les grands monolithes, et une analyse de 2024 portant sur 10 000 projets Python open source a révélé que <strong>34 % contenaient au moins une dépendance circulaire forte</strong> affectant l'isolation des tests.</p>

<p>Le code mort est tout aussi insidieux. Des recherches de l'Université de Zurich en 2025 ont estimé que la base de code d'entreprise moyenne contient <strong>12 à 18 % de code mort</strong> en volume. Ce code mort augmente les temps de compilation, élargit la surface d'attaque de sécurité, déroute les développeurs qui lisent le code et gonfle la taille des bundles pour les applications frontend.</p>

<p>Les linters traditionnels ne détectent rien de tout cela. Un fichier contenant du code mort est syntaxiquement valide. Une dépendance circulaire implique des fichiers qui sont individuellement corrects. Les problèmes ne deviennent visibles que lorsqu'on examine la base de code dans son ensemble.</p>

<h2 id="how-aigiscode-fits-the-ecosystem">Comment AigisCode s'intègre dans l'écosystème</h2>

<p>AigisCode ne remplace pas votre linter. Il le complète. Considérez les linters comme votre correcteur orthographique et AigisCode comme votre éditeur structurel. ESLint s'assure que votre JavaScript suit des modèles cohérents. AigisCode s'assure que vos modules ne forment pas de cycles de dépendances qui rendent votre application impossible à déployer de manière incrémentale.</p>

<p>L'outil est open source sous la licence MIT et installable from GitHub or via cargo. Une seule commande, <code>aigiscode analyze /path/to/project</code>, exécute le pipeline complet en six étapes. La sortie est un rapport JSON lisible par machine que les humains et les agents IA peuvent consommer. Les fichiers de politique permettent aux équipes de personnaliser le comportement de détection par projet, en supprimant les faux positifs et en encodant les connaissances spécifiques au projet.</p>

<p>Les profils de plugins intégrés pour Django, Laravel et WordPress gèrent les modèles spécifiques aux frameworks dès l'installation. Par exemple, le plugin Django reconnaît que les classes de modèles référencées dans les migrations ne sont pas du code mort, même si aucun fichier Python ne les importe directement. Le plugin Laravel comprend les liaisons des fournisseurs de services et les accesseurs de façades. Ces types d'ajustements tenant compte du framework éliminent des catégories entières de faux positifs sans nécessiter de configuration manuelle.</p>

<h2 id="looking-ahead">Perspectives d'avenir</h2>

<p>La trajectoire est claire. À mesure que les bases de code grandissent et que les agents IA prennent en charge davantage de travail de développement, le besoin d'une analyse structurelle allant au-delà du linting au niveau des fichiers ne fera qu'augmenter. Nous nous dirigeons vers un monde où chaque pull request est évaluée non seulement pour le style du code et la couverture de tests, mais aussi pour son impact sur le graphe de dépendances, sa contribution à la dette technique et son alignement avec l'architecture prévue.</p>

<p>L'analyse de code par IA est le pont entre les linters d'aujourd'hui et les systèmes de qualité de code entièrement automatisés de demain. Les équipes qui l'adoptent maintenant auront des architectures plus propres, une intégration plus rapide des nouveaux développeurs et moins de surprises en production. Les outils existent. Les pratiques sont éprouvées. La question n'est plus de savoir si l'analyse de code par IA est importante. C'est de savoir si votre équipe peut se permettre de travailler sans.</p>
`,
      es: `
<p>El año 2026 ha traído un cambio fundamental en la forma en que pensamos sobre la calidad del código. Hemos dejado atrás la era en la que ejecutar un linter se consideraba diligencia suficiente. Hoy en día, las bases de código abarcan cientos de miles de archivos, los microservicios se comunican a través de contratos invisibles y los agentes de codificación IA generan código a un ritmo que la revisión humana por sí sola no puede igualar. En este panorama, el análisis de código impulsado por IA no es un lujo. Es una necesidad.</p>

<h2 id="the-limitations-of-traditional-linters">Las limitaciones de los linters tradicionales</h2>

<p>Los linters tradicionales como ESLint, Pylint y PHPStan son excelentes en lo que hacen. Detectan errores de sintaxis, imponen convenciones de estilo, señalan variables no utilizadas y garantizan un formato consistente. Pero operan dentro de una restricción fundamental: analizan los archivos de forma aislada.</p>

<p>Considere una aplicación Django grande con 400 módulos Python. Pylint puede decirle que una importación particular no se usa <em>dentro de un solo archivo</em>. No puede decirle que el módulo A depende del módulo B, que depende del módulo C, que a su vez depende del módulo A, creando una dependencia circular que hace imposible probar todo el subsistema de forma aislada. No puede decirle que una clase utilitaria en <code>utils/helpers.py</code> es importada por 47 archivos diferentes, convirtiéndola en un cuello de botella peligroso donde un solo cambio se propaga de manera impredecible. No puede decirle que un directorio entero de clases de controladores ha quedado huérfano desde la última refactorización, acumulando polvo y confundiendo a cada nuevo desarrollador que lo encuentra.</p>

<p>Estos son <strong>problemas arquitectónicos</strong>. Existen en las relaciones entre archivos, no dentro de ellos. Y son los problemas que más tiempo cuestan a los equipos de ingeniería, causan la mayoría de los incidentes en producción y generan la mayor frustración durante la incorporación de nuevos miembros.</p>

<h2 id="what-ai-brings-to-static-analysis">Lo que la IA aporta al análisis estático</h2>

<p>El análisis estático impulsado por IA opera a un nivel de abstracción diferente. En lugar de examinar archivos individuales, construye un <strong>grafo de dependencias</strong> de toda su base de código. Comprende qué módulos importan cuáles, cómo fluyen los símbolos entre archivos y dónde están los puntos de presión estructural.</p>

<p>Aquí es donde herramientas como AigisCode se diferencian de los enfoques tradicionales. AigisCode ejecuta un pipeline de seis etapas: Index, Graph, Detect, Rules, AI Review y Report. Las tres primeras etapas son puramente deterministas. Tree-sitter analiza los archivos fuente en PHP, Python, TypeScript, JavaScript y Vue. petgraph construye un grafo de dependencias completo. Los detectores identifican candidatos para dependencias circulares, código muerto, valores codificados en duro y violaciones arquitectónicas.</p>

<p>La IA entra en la quinta etapa. Después de que los detectores deterministas han producido candidatos con niveles de confianza, la etapa de revisión IA clasifica los hallazgos como verdaderos positivos, falsos positivos o necesita-contexto. Este enfoque híbrido es fundamental. Las etapas deterministas garantizan la reproducibilidad y la explicabilidad. La etapa de IA maneja los matices que las heurísticas puras no pueden capturar.</p>

<h2 id="the-rise-of-ai-coding-agents">El auge de los agentes de codificación IA</h2>

<p>Quizás la tendencia más significativa de 2025 y 2026 ha sido la proliferación de agentes de codificación IA. Herramientas como Claude Code, GitHub Copilot Workspace y los agentes Codex ahora pueden implementar funcionalidades, corregir errores y refactorizar código de forma autónoma. Pero estos agentes necesitan retroalimentación estructurada y legible por máquinas sobre la salud de la base de código para operar eficazmente.</p>

<p>La salida de un linter es útil para un agente IA, pero limitada. Un agente puede corregir una advertencia de importación no utilizada. Pero, ¿puede entender que resolver una dependencia circular entre tres módulos requiere reestructurar las abstracciones compartidas? ¿Puede decidir cuál de los 23 hallazgos de código muerto abordar primero basándose en el perfil de riesgo del código circundante?</p>

<p>Aquí es donde las herramientas de análisis impulsadas por IA se convierten en el sistema sensorial de los agentes IA. AigisCode genera un informe JSON estructurado en <code>.aigiscode/deterministic-analysis.json</code> que un agente puede analizar directamente. El informe incluye niveles de severidad, puntuaciones de confianza, rutas de archivos y explicaciones. Un agente puede leer este informe, clasificar los hallazgos por confianza y comenzar a corregir automáticamente los problemas de mayor impacto.</p>

<h2 id="the-numbers-tell-the-story">Los números cuentan la historia</h2>

<p>El impacto de los problemas arquitectónicos está bien documentado. Un estudio de 2025 de Stripe encontró que los desarrolladores dedican aproximadamente el <strong>42 % de su tiempo</strong> a lidiar con deuda técnica y mantenimiento, frente al 33 % en 2018. Las dependencias circulares son una causa principal del "infierno de dependencias" en grandes monolitos, y un análisis de 2024 de 10 000 proyectos Python de código abierto encontró que el <strong>34 % contenía al menos una dependencia circular fuerte</strong> que afectaba el aislamiento de las pruebas.</p>

<p>El código muerto es igualmente insidioso. Investigaciones de la Universidad de Zúrich en 2025 estimaron que la base de código empresarial promedio contiene entre un <strong>12 y un 18 % de código muerto</strong> en volumen. Este código muerto aumenta los tiempos de compilación, amplía la superficie de ataque de seguridad, confunde a los desarrolladores que leen el código e infla el tamaño de los bundles para aplicaciones frontend.</p>

<p>Los linters tradicionales no detectan nada de esto. Un archivo que contiene código muerto es sintácticamente válido. Una dependencia circular involucra archivos que son individualmente correctos. Los problemas solo se hacen visibles cuando se observa la base de código en su conjunto.</p>

<h2 id="how-aigiscode-fits-the-ecosystem">Cómo AigisCode encaja en el ecosistema</h2>

<p>AigisCode no reemplaza su linter. Lo complementa. Piense en los linters como su corrector ortográfico y en AigisCode como su editor estructural. ESLint asegura que su JavaScript siga patrones consistentes. AigisCode asegura que sus módulos no formen ciclos de dependencias que hagan imposible desplegar su aplicación de forma incremental.</p>

<p>La herramienta es de código abierto bajo la licencia MIT e instalable vía pip. Un solo comando, <code>aigiscode analyze /path/to/project</code>, ejecuta el pipeline completo de seis etapas. La salida es un informe JSON legible por máquinas que tanto humanos como agentes IA pueden consumir. Los archivos de políticas permiten a los equipos personalizar el comportamiento de detección por proyecto, suprimiendo falsos positivos y codificando conocimiento específico del proyecto.</p>

<p>Los perfiles de plugins integrados para Django, Laravel y WordPress manejan patrones específicos de frameworks de forma predeterminada. Por ejemplo, el plugin de Django reconoce que las clases de modelos referenciadas en migraciones no son código muerto, incluso si ningún archivo Python las importa directamente. El plugin de Laravel comprende los enlaces de proveedores de servicios y los accesores de fachadas. Este tipo de ajustes conscientes del framework eliminan categorías enteras de falsos positivos sin requerir configuración manual.</p>

<h2 id="looking-ahead">Mirando hacia adelante</h2>

<p>La trayectoria es clara. A medida que las bases de código crecen y los agentes IA asumen más trabajo de desarrollo, la necesidad de un análisis estructural que vaya más allá del linting a nivel de archivo solo aumentará. Nos dirigimos hacia un mundo donde cada pull request se evalúa no solo por el estilo del código y la cobertura de pruebas, sino por su impacto en el grafo de dependencias, su contribución a la deuda técnica y su alineación con la arquitectura prevista.</p>

<p>El análisis de código impulsado por IA es el puente entre los linters de hoy y los sistemas de calidad de código completamente automatizados del mañana. Los equipos que lo adopten ahora tendrán arquitecturas más limpias, una incorporación más rápida y menos sorpresas en producción. Las herramientas existen. Los patrones están probados. La pregunta ya no es si el análisis de código con IA importa. Es si su equipo puede permitirse trabajar sin él.</p>
`,
      zh: `
<p>2026 年带来了我们思考代码质量方式的根本性转变。我们已经走过了仅仅运行一个 linter 就被认为是充分尽职的时代。如今，代码库跨越数十万个文件，微服务通过不可见的契约进行通信，AI 编程代理以人工审查无法匹敌的速度生成代码。在这种环境下，AI 驱动的代码分析不是奢侈品，而是必需品。</p>

<h2 id="the-limitations-of-traditional-linters">传统 Linter 的局限性</h2>

<p>ESLint、Pylint 和 PHPStan 等传统 linter 在其职能范围内表现出色。它们能捕获语法错误、强制执行样式约定、标记未使用的变量并确保格式一致。但它们在一个根本性约束下运行：它们孤立地分析文件。</p>

<p>以一个拥有 400 个 Python 模块的大型 Django 应用为例。Pylint 可以告诉你某个特定的导入在<em>单个文件内</em>未被使用。它无法告诉你模块 A 依赖模块 B，模块 B 依赖模块 C，模块 C 又依赖回模块 A，形成循环依赖，使得整个子系统无法独立测试。它无法告诉你 <code>utils/helpers.py</code> 中的一个工具类被 47 个不同的文件导入，使其成为一个危险的瓶颈，一个变更会不可预测地级联传播。它无法告诉你一整个目录的处理器类自上次重构以来已成为孤儿，一直在那里积灰并让每个遇到它的新开发者感到困惑。</p>

<p>这些都是<strong>架构问题</strong>。它们存在于文件之间的关系中，而非文件内部。它们是耗费工程团队最多时间、造成最多生产事故、在新人入职时带来最多挫败感的问题。</p>

<h2 id="what-ai-brings-to-static-analysis">AI 为静态分析带来了什么</h2>

<p>AI 驱动的静态分析在不同的抽象层级上运作。它不是检查单个文件，而是构建整个代码库的<strong>依赖图</strong>。它理解哪些模块导入哪些模块、符号如何在文件之间流动，以及结构性压力点在哪里。</p>

<p>这正是 AigisCode 等工具与传统方法的不同之处。AigisCode 运行一个六阶段流水线：Index、Graph、Detect、Rules、AI Review 和 Report。前三个阶段是纯确定性的。Tree-sitter 解析 PHP、Python、TypeScript、JavaScript 和 Vue 的源文件。petgraph 构建完整的依赖图。检测器识别循环依赖、死代码、硬编码值和架构违规的候选项。</p>

<p>AI 在第五阶段介入。在确定性检测器生成带有置信度的候选项之后，AI 审查阶段将发现分类为真阳性、假阳性或需要上下文。这种混合方法至关重要。确定性阶段确保可重复性和可解释性。AI 阶段处理纯启发式方法无法捕获的细微差别。</p>

<h2 id="the-rise-of-ai-coding-agents">AI 编程代理的崛起</h2>

<p>2025 年和 2026 年最重要的趋势也许就是 AI 编程代理的普及。Claude Code、GitHub Copilot Workspace 和 Codex 代理等工具现在可以自主实现功能、修复 bug 和重构代码。但这些代理需要关于代码库健康状况的结构化、机器可读的反馈才能有效运行。</p>

<p>Linter 的输出对 AI 代理有用，但有限。代理可以修复一个未使用的导入警告。但它能理解修复三个模块之间的循环依赖需要重构共享的抽象吗？它能根据周围代码的风险特征决定 23 个死代码发现中优先处理哪一个吗？</p>

<p>这正是 AI 驱动的分析工具成为 AI 代理感知系统的地方。AigisCode 在 <code>.aigiscode/deterministic-analysis.json</code> 输出结构化的 JSON 报告，代理可以直接解析。报告包含严重级别、置信度评分、文件路径和说明。代理可以读取此报告，按置信度对发现进行分类，并自动开始修复影响最大的问题。</p>

<h2 id="the-numbers-tell-the-story">数据说明一切</h2>

<p>架构问题的影响已有充分记录。Stripe 在 2025 年的一项研究发现，开发人员估计将<strong>42% 的时间</strong>花在处理技术债务和维护上，高于 2018 年的 33%。循环依赖是大型单体应用中"依赖地狱"的主要原因，2024 年对 10,000 个开源 Python 项目的分析发现，<strong>34% 至少包含一个影响测试隔离的强循环依赖</strong>。</p>

<p>死代码同样阴险。苏黎世大学 2025 年的研究估计，企业级代码库平均包含<strong>12-18% 的死代码</strong>。这些死代码增加了构建时间、扩大了安全攻击面、让阅读代码的开发者感到困惑，并且膨胀了前端应用的打包体积。</p>

<p>传统 linter 对此毫无察觉。包含死代码的文件在语法上是有效的。循环依赖涉及的文件单独来看都是正确的。只有当你将代码库作为整体来看时，问题才会显现。</p>

<h2 id="how-aigiscode-fits-the-ecosystem">AigisCode 如何融入生态系统</h2>

<p>AigisCode 不会取代你的 linter，而是补充它。把 linter 想象成你的拼写检查器，把 AigisCode 想象成你的结构编辑器。ESLint 确保你的 JavaScript 遵循一致的模式。AigisCode 确保你的模块不会形成让应用无法增量部署的依赖循环。</p>

<p>该工具是基于 MIT 许可证的开源软件，可通过 pip 安装。一条命令 <code>aigiscode analyze /path/to/project</code> 即可运行完整的六阶段流水线。输出是人类和 AI 代理都可以使用的机器可读 JSON 报告。策略文件允许团队按项目自定义检测行为，抑制误报并编码项目特定的知识。</p>

<p>内置的 Django、Laravel 和 WordPress 插件配置文件开箱即用地处理框架特定模式。例如，Django 插件识别到迁移中引用的模型类不是死代码，即使没有 Python 文件直接导入它们。Laravel 插件理解服务提供者绑定和门面访问器。这类框架感知的调整消除了整类误报，无需手动配置。</p>

<h2 id="looking-ahead">展望未来</h2>

<p>趋势很明确。随着代码库越来越大、AI 代理承担更多开发工作，超越文件级 linting 的结构分析需求只会增加。我们正走向一个世界，每个 pull request 不仅要评估代码风格和测试覆盖率，还要评估其对依赖图的影响、对技术债务的贡献以及与预期架构的一致性。</p>

<p>AI 驱动的代码分析是连接当今 linter 和明天全自动代码质量系统的桥梁。现在采用它的团队将拥有更干净的架构、更快的新人入职和更少的生产意外。工具已经存在。模式已经验证。问题不再是 AI 代码分析是否重要，而是你的团队是否能承担没有它的代价。</p>
`,
      hi: `
<p>वर्ष 2026 ने कोड गुणवत्ता के बारे में हमारी सोच में एक मौलिक बदलाव लाया है। हम उस युग से आगे बढ़ चुके हैं जब एक लिंटर चलाना पर्याप्त सावधानी माना जाता था। आज, कोडबेस लाखों फाइलों में फैले हुए हैं, माइक्रोसर्विसेज अदृश्य कॉन्ट्रैक्ट्स के माध्यम से संवाद करती हैं, और AI कोडिंग एजेंट ऐसी गति से कोड उत्पन्न करते हैं जिसकी बराबरी अकेले मानव समीक्षा नहीं कर सकती। इस परिदृश्य में, AI-संचालित कोड विश्लेषण कोई विलासिता नहीं है। यह एक आवश्यकता है।</p>

<h2 id="the-limitations-of-traditional-linters">पारंपरिक लिंटर्स की सीमाएं</h2>

<p>ESLint, Pylint और PHPStan जैसे पारंपरिक लिंटर्स अपने काम में उत्कृष्ट हैं। वे सिंटैक्स त्रुटियों को पकड़ते हैं, स्टाइल कन्वेंशन लागू करते हैं, अनुपयोगी वेरिएबल्स को चिह्नित करते हैं, और सुसंगत फॉर्मेटिंग सुनिश्चित करते हैं। लेकिन वे एक मूलभूत सीमा के भीतर काम करते हैं: वे फाइलों का अलग-अलग विश्लेषण करते हैं।</p>

<p>400 Python मॉड्यूल वाले एक बड़े Django एप्लिकेशन पर विचार करें। Pylint आपको बता सकता है कि कोई विशेष import <em>एक फाइल के भीतर</em> अनुपयोगी है। यह आपको नहीं बता सकता कि मॉड्यूल A मॉड्यूल B पर निर्भर है, जो मॉड्यूल C पर निर्भर है, जो वापस मॉड्यूल A पर निर्भर है, जिससे एक चक्रीय निर्भरता बनती है जो पूरे उपतंत्र को अलग से परीक्षण करना असंभव बना देती है। यह आपको नहीं बता सकता कि <code>utils/helpers.py</code> में एक यूटिलिटी क्लास को 47 अलग-अलग फाइलों द्वारा आयात किया जाता है, जो इसे एक खतरनाक बॉटलनेक बना देता है जहां एक भी बदलाव अप्रत्याशित रूप से फैलता है। यह आपको नहीं बता सकता कि हैंडलर क्लासेज की एक पूरी डायरेक्टरी अंतिम रीफैक्टरिंग के बाद से अनाथ हो गई है, वहां बैठी धूल जमा कर रही है और हर नए डेवलपर को भ्रमित कर रही है।</p>

<p>ये <strong>आर्किटेक्चरल समस्याएं</strong> हैं। ये फाइलों के बीच के संबंधों में मौजूद हैं, उनके भीतर नहीं। और ये वही समस्याएं हैं जो इंजीनियरिंग टीमों का सबसे अधिक समय खर्च करती हैं, सबसे अधिक प्रोडक्शन इंसिडेंट्स का कारण बनती हैं, और ऑनबोर्डिंग के दौरान सबसे अधिक निराशा पैदा करती हैं।</p>

<h2 id="what-ai-brings-to-static-analysis">AI स्टैटिक विश्लेषण में क्या लाता है</h2>

<p>AI-संचालित स्टैटिक विश्लेषण अमूर्तता के एक अलग स्तर पर काम करता है। व्यक्तिगत फाइलों की जांच करने के बजाय, यह आपके पूरे कोडबेस का एक <strong>डिपेंडेंसी ग्राफ</strong> बनाता है। यह समझता है कि कौन से मॉड्यूल किसे आयात करते हैं, सिंबल फाइलों के बीच कैसे प्रवाहित होते हैं, और संरचनात्मक दबाव बिंदु कहां हैं।</p>

<p>यहीं पर AigisCode जैसे टूल्स पारंपरिक दृष्टिकोणों से भिन्न हैं। AigisCode एक छह-चरणीय पाइपलाइन चलाता है: Index, Graph, Detect, Rules, AI Review, और Report। पहले तीन चरण पूरी तरह से नियतात्मक हैं। Tree-sitter PHP, Python, TypeScript, JavaScript और Vue में स्रोत फाइलों को पार्स करता है। petgraph एक पूर्ण डिपेंडेंसी ग्राफ बनाता है। डिटेक्टर चक्रीय निर्भरताओं, डेड कोड, हार्डवायर्ड मानों और आर्किटेक्चरल उल्लंघनों के लिए उम्मीदवारों की पहचान करते हैं।</p>

<p>AI पांचवें चरण में प्रवेश करता है। नियतात्मक डिटेक्टरों द्वारा विश्वास स्तरों के साथ उम्मीदवार तैयार करने के बाद, AI समीक्षा चरण निष्कर्षों को सही सकारात्मक, गलत सकारात्मक, या संदर्भ-आवश्यक के रूप में वर्गीकृत करता है। यह हाइब्रिड दृष्टिकोण महत्वपूर्ण है। नियतात्मक चरण पुनरुत्पादनीयता और व्याख्या योग्यता सुनिश्चित करते हैं। AI चरण उस बारीकियों को संभालता है जो शुद्ध अनुमान नहीं पकड़ सकते।</p>

<h2 id="the-rise-of-ai-coding-agents">AI कोडिंग एजेंट्स का उदय</h2>

<p>संभवतः 2025 और 2026 का सबसे महत्वपूर्ण रुझान AI कोडिंग एजेंट्स का प्रसार रहा है। Claude Code, GitHub Copilot Workspace, और Codex एजेंट्स जैसे टूल्स अब स्वायत्त रूप से फीचर्स लागू कर सकते हैं, बग ठीक कर सकते हैं, और कोड को रीफैक्टर कर सकते हैं। लेकिन इन एजेंट्स को प्रभावी ढंग से काम करने के लिए कोडबेस स्वास्थ्य के बारे में संरचित, मशीन-पठनीय प्रतिक्रिया की आवश्यकता है।</p>

<p>एक लिंटर का आउटपुट AI एजेंट के लिए उपयोगी है, लेकिन सीमित है। एक एजेंट अनुपयोगी import की चेतावनी को ठीक कर सकता है। लेकिन क्या यह समझ सकता है कि तीन मॉड्यूलों के बीच चक्रीय निर्भरता को ठीक करने के लिए साझा अमूर्तताओं की पुनर्संरचना की आवश्यकता है? क्या यह तय कर सकता है कि आसपास के कोड के जोखिम प्रोफाइल के आधार पर 23 डेड कोड निष्कर्षों में से पहले किसे संबोधित करना चाहिए?</p>

<p>यहीं पर AI-संचालित विश्लेषण उपकरण AI एजेंट्स के लिए संवेदी प्रणाली बन जाते हैं। AigisCode <code>.aigiscode/deterministic-analysis.json</code> पर एक संरचित JSON रिपोर्ट उत्पन्न करता है जिसे एजेंट सीधे पार्स कर सकता है। रिपोर्ट में गंभीरता स्तर, विश्वास स्कोर, फाइल पथ और स्पष्टीकरण शामिल हैं। एजेंट इस रिपोर्ट को पढ़ सकता है, विश्वास के अनुसार निष्कर्षों को ट्राइएज कर सकता है, और स्वचालित रूप से सबसे प्रभावशाली मुद्दों को ठीक करना शुरू कर सकता है।</p>

<h2 id="the-numbers-tell-the-story">संख्याएं कहानी बयान करती हैं</h2>

<p>आर्किटेक्चरल समस्याओं का प्रभाव अच्छी तरह से प्रलेखित है। Stripe के 2025 के एक अध्ययन में पाया गया कि डेवलपर्स अपने समय का अनुमानित <strong>42%</strong> तकनीकी ऋण और रखरखाव से निपटने में बिताते हैं, जो 2018 में 33% से बढ़ गया है। चक्रीय निर्भरताएं बड़े मोनोलिथ्स में "निर्भरता नरक" का प्रमुख कारण हैं, और 10,000 ओपन-सोर्स Python प्रोजेक्ट्स के 2024 के विश्लेषण में पाया गया कि <strong>34% में कम से कम एक मजबूत चक्रीय निर्भरता</strong> थी जो परीक्षण अलगाव को प्रभावित करती थी।</p>

<p>डेड कोड भी उतना ही कपटी है। 2025 में ज्यूरिख विश्वविद्यालय के शोध ने अनुमान लगाया कि औसत एंटरप्राइज कोडबेस में मात्रा के अनुसार <strong>12-18% डेड कोड</strong> होता है। यह डेड कोड बिल्ड समय बढ़ाता है, सुरक्षा हमले की सतह का विस्तार करता है, कोड पढ़ने वाले डेवलपर्स को भ्रमित करता है, और फ्रंटएंड एप्लिकेशन के लिए बंडल आकार बढ़ाता है।</p>

<p>पारंपरिक लिंटर्स इनमें से कुछ भी नहीं पकड़ते। डेड कोड वाली फाइल सिंटैक्टिक रूप से वैध है। चक्रीय निर्भरता में ऐसी फाइलें शामिल हैं जो व्यक्तिगत रूप से सही हैं। समस्याएं तभी दिखाई देती हैं जब आप कोडबेस को समग्र रूप से देखते हैं।</p>

<h2 id="how-aigiscode-fits-the-ecosystem">AigisCode इकोसिस्टम में कैसे फिट होता है</h2>

<p>AigisCode आपके लिंटर को प्रतिस्थापित नहीं करता। यह उसका पूरक है। लिंटर्स को अपने वर्तनी-जांचक के रूप में और AigisCode को अपने संरचनात्मक संपादक के रूप में सोचें। ESLint सुनिश्चित करता है कि आपका JavaScript सुसंगत पैटर्न का पालन करे। AigisCode सुनिश्चित करता है कि आपके मॉड्यूल ऐसे निर्भरता चक्र न बनाएं जो आपके एप्लिकेशन को क्रमिक रूप से तैनात करना असंभव बना दें।</p>

<p>यह टूल MIT लाइसेंस के तहत ओपन सोर्स है। एक ही कमांड, <code>aigiscode analyze /path/to/project</code>, पूरी छह-चरणीय पाइपलाइन चलाता है। आउटपुट एक मशीन-पठनीय JSON रिपोर्ट है जिसे मनुष्य और AI एजेंट दोनों उपयोग कर सकते हैं। पॉलिसी फाइलें टीमों को प्रत्येक प्रोजेक्ट के लिए डिटेक्शन व्यवहार को अनुकूलित करने, गलत सकारात्मक को दबाने और प्रोजेक्ट-विशिष्ट ज्ञान को एन्कोड करने की अनुमति देती हैं।</p>

<p>Django, Laravel और WordPress के लिए अंतर्निहित प्लगइन प्रोफाइल फ्रेमवर्क-विशिष्ट पैटर्न को बिना किसी अतिरिक्त कॉन्फ़िगरेशन के संभालते हैं। उदाहरण के लिए, Django प्लगइन पहचानता है कि माइग्रेशन में संदर्भित मॉडल क्लासेज डेड कोड नहीं हैं, भले ही कोई Python फाइल उन्हें सीधे आयात न करे। Laravel प्लगइन सर्विस प्रोवाइडर बाइंडिंग और फेसेड एक्सेसर को समझता है। इस प्रकार के फ्रेमवर्क-जागरूक समायोजन मैनुअल कॉन्फ़िगरेशन की आवश्यकता के बिना गलत सकारात्मक की पूरी श्रेणियों को समाप्त कर देते हैं।</p>

<h2 id="looking-ahead">भविष्य की ओर</h2>

<p>प्रक्षेपवक्र स्पष्ट है। जैसे-जैसे कोडबेस बड़े होते हैं और AI एजेंट अधिक विकास कार्य संभालते हैं, फाइल-स्तरीय लिंटिंग से परे संरचनात्मक विश्लेषण की आवश्यकता केवल बढ़ेगी। हम ऐसी दुनिया की ओर बढ़ रहे हैं जहां हर पुल रिक्वेस्ट का मूल्यांकन न केवल कोड स्टाइल और टेस्ट कवरेज के लिए किया जाता है, बल्कि डिपेंडेंसी ग्राफ पर इसके प्रभाव, तकनीकी ऋण में इसके योगदान, और इच्छित आर्किटेक्चर के साथ इसके संरेखण के लिए भी।</p>

<p>AI-संचालित कोड विश्लेषण आज के लिंटर्स और कल की पूरी तरह स्वचालित कोड गुणवत्ता प्रणालियों के बीच का सेतु है। जो टीमें इसे अभी अपनाती हैं, उनके पास स्वच्छ आर्किटेक्चर, तेज ऑनबोर्डिंग और कम प्रोडक्शन आश्चर्य होंगे। उपकरण मौजूद हैं। पैटर्न सिद्ध हैं। सवाल अब यह नहीं है कि AI कोड विश्लेषण मायने रखता है या नहीं। सवाल यह है कि क्या आपकी टीम इसके बिना काम करने का जोखिम उठा सकती है।</p>
`,
      pt: `
<p>O ano de 2026 trouxe uma mudança fundamental na forma como pensamos sobre qualidade de código. Superamos a era em que executar um linter era considerado diligência suficiente. Hoje, as bases de código abrangem centenas de milhares de arquivos, microsserviços comunicam-se através de contratos invisíveis, e agentes de codificação com IA geram código a um ritmo que a revisão humana sozinha não consegue acompanhar. Nesse cenário, a análise de código com IA não é um luxo. É uma necessidade.</p>

<h2 id="the-limitations-of-traditional-linters">As Limitações dos Linters Tradicionais</h2>

<p>Linters tradicionais como ESLint, Pylint e PHPStan são excelentes no que fazem. Eles detectam erros de sintaxe, aplicam convenções de estilo, sinalizam variáveis não utilizadas e garantem formatação consistente. Mas operam dentro de uma restrição fundamental: analisam arquivos isoladamente.</p>

<p>Considere uma grande aplicação Django com 400 módulos Python. O Pylint pode dizer que um determinado import não é utilizado <em>dentro de um único arquivo</em>. Ele não pode dizer que o módulo A depende do módulo B, que depende do módulo C, que depende de volta do módulo A, criando uma dependência circular que torna todo o subsistema impossível de testar isoladamente. Não pode dizer que uma classe utilitária em <code>utils/helpers.py</code> é importada por 47 arquivos diferentes, tornando-a um gargalo perigoso onde uma única alteração se propaga de forma imprevisível. Não pode dizer que um diretório inteiro de classes handler ficou órfão desde a última refatoração, acumulando poeira e confundindo cada novo desenvolvedor que o encontra.</p>

<p>Estes são <strong>problemas arquiteturais</strong>. Existem nas relações entre arquivos, não dentro deles. E são os problemas que custam mais tempo às equipes de engenharia, causam mais incidentes em produção e criam mais frustração durante a integração de novos membros.</p>

<h2 id="what-ai-brings-to-static-analysis">O que a IA Traz à Análise Estática</h2>

<p>A análise estática com IA opera num nível diferente de abstração. Em vez de examinar arquivos individuais, ela constrói um <strong>grafo de dependências</strong> de toda a sua base de código. Compreende quais módulos importam quais, como os símbolos fluem entre arquivos e onde estão os pontos de pressão estrutural.</p>

<p>É aqui que ferramentas como o AigisCode diferem das abordagens tradicionais. O AigisCode executa um pipeline de seis estágios: Index, Graph, Detect, Rules, AI Review e Report. Os três primeiros estágios são puramente determinísticos. O Tree-sitter analisa arquivos fonte em PHP, Python, TypeScript, JavaScript e Vue. O petgraph constrói um grafo completo de dependências. Os detectores identificam candidatos para dependências circulares, código morto, valores hardcoded e violações arquiteturais.</p>

<p>A IA entra no estágio cinco. Depois que os detectores determinísticos produziram candidatos com níveis de confiança, o estágio de revisão por IA classifica as descobertas como verdadeiros positivos, falsos positivos ou necessitando de contexto. Esta abordagem híbrida é crítica. Os estágios determinísticos garantem reprodutibilidade e explicabilidade. O estágio de IA lida com as nuances que heurísticas puras não conseguem capturar.</p>

<h2 id="the-rise-of-ai-coding-agents">A Ascensão dos Agentes de Codificação com IA</h2>

<p>Talvez a tendência mais significativa de 2025 e 2026 tenha sido a proliferação de agentes de codificação com IA. Ferramentas como Claude Code, GitHub Copilot Workspace e agentes Codex agora podem implementar funcionalidades, corrigir bugs e refatorar código de forma autônoma. Mas esses agentes precisam de feedback estruturado e legível por máquina sobre a saúde da base de código para operar efetivamente.</p>

<p>A saída de um linter é útil para um agente de IA, mas limitada. Um agente pode corrigir um aviso de import não utilizado. Mas pode entender que corrigir uma dependência circular entre três módulos requer reestruturar as abstrações compartilhadas? Pode decidir qual das 23 descobertas de código morto abordar primeiro com base no perfil de risco do código circundante?</p>

<p>É aqui que as ferramentas de análise com IA se tornam o sistema sensorial para agentes de IA. O AigisCode gera um relatório JSON estruturado em <code>.aigiscode/deterministic-analysis.json</code> que um agente pode analisar diretamente. O relatório inclui níveis de severidade, pontuações de confiança, caminhos de arquivos e explicações. Um agente pode ler este relatório, triar descobertas por confiança e começar a corrigir os problemas mais impactantes automaticamente.</p>

<h2 id="the-numbers-tell-the-story">Os Números Contam a História</h2>

<p>O impacto dos problemas arquiteturais está bem documentado. Um estudo de 2025 da Stripe descobriu que os desenvolvedores gastam cerca de <strong>42% do seu tempo</strong> lidando com dívida técnica e manutenção, acima dos 33% em 2018. Dependências circulares são uma causa principal do "inferno de dependências" em grandes monolitos, e uma análise de 2024 de 10.000 projetos Python de código aberto descobriu que <strong>34% continham pelo menos uma dependência circular forte</strong> que afetava o isolamento de testes.</p>

<p>Código morto é igualmente insidioso. Uma pesquisa da Universidade de Zurique em 2025 estimou que a base de código empresarial média contém <strong>12-18% de código morto</strong> por volume. Este código morto aumenta os tempos de compilação, expande a superfície de ataque de segurança, confunde desenvolvedores que leem o código e infla o tamanho dos bundles para aplicações frontend.</p>

<p>Linters tradicionais não detectam nada disso. Um arquivo contendo código morto é sintaticamente válido. Uma dependência circular envolve arquivos que são individualmente corretos. Os problemas só se tornam visíveis quando se olha para a base de código como um todo.</p>

<h2 id="how-aigiscode-fits-the-ecosystem">Como o AigisCode se Encaixa no Ecossistema</h2>

<p>O AigisCode não substitui o seu linter. Ele o complementa. Pense nos linters como o seu corretor ortográfico e no AigisCode como o seu editor estrutural. O ESLint garante que o seu JavaScript siga padrões consistentes. O AigisCode garante que os seus módulos não formem ciclos de dependência que tornem a sua aplicação impossível de implantar incrementalmente.</p>

<p>A ferramenta é de código aberto sob a licença MIT. Um único comando, <code>aigiscode analyze /path/to/project</code>, executa o pipeline completo de seis estágios. A saída é um relatório JSON legível por máquina que tanto humanos quanto agentes de IA podem consumir. Arquivos de política permitem que as equipes personalizem o comportamento de detecção por projeto, suprimindo falsos positivos e codificando conhecimento específico do projeto.</p>

<p>Perfis de plugins integrados para Django, Laravel e WordPress lidam com padrões específicos de framework sem configuração adicional. Por exemplo, o plugin Django reconhece que classes de modelo referenciadas em migrações não são código morto, mesmo que nenhum arquivo Python as importe diretamente. O plugin Laravel compreende as vinculações de service provider e os acessores de facade. Esses tipos de ajustes conscientes do framework eliminam categorias inteiras de falsos positivos sem exigir configuração manual.</p>

<h2 id="looking-ahead">Olhando para o Futuro</h2>

<p>A trajetória é clara. À medida que as bases de código crescem e os agentes de IA assumem mais trabalho de desenvolvimento, a necessidade de análise estrutural que vá além do linting no nível de arquivo só vai aumentar. Estamos caminhando para um mundo onde cada pull request é avaliado não apenas pelo estilo de código e cobertura de testes, mas pelo seu impacto no grafo de dependências, sua contribuição para a dívida técnica e seu alinhamento com a arquitetura pretendida.</p>

<p>A análise de código com IA é a ponte entre os linters de hoje e os sistemas de qualidade de código totalmente automatizados de amanhã. As equipes que a adotarem agora terão arquiteturas mais limpas, integração mais rápida e menos surpresas em produção. As ferramentas existem. Os padrões estão comprovados. A questão não é mais se a análise de código com IA importa. É se a sua equipe pode se dar ao luxo de trabalhar sem ela.</p>
`,
      ar: `
<p>جلب عام 2026 تحولاً جوهرياً في طريقة تفكيرنا حول جودة الشيفرة البرمجية. لقد تجاوزنا الحقبة التي كان فيها تشغيل أداة فحص الشيفرة (linter) يُعتبر عناية كافية. اليوم، تمتد قواعد الشيفرة عبر مئات الآلاف من الملفات، وتتواصل الخدمات المصغرة من خلال عقود غير مرئية، وينتج وكلاء البرمجة بالذكاء الاصطناعي شيفرة بوتيرة لا يمكن للمراجعة البشرية وحدها مجاراتها. في هذا المشهد، تحليل الشيفرة المدعوم بالذكاء الاصطناعي ليس ترفاً. إنه ضرورة.</p>

<h2 id="the-limitations-of-traditional-linters">حدود أدوات الفحص التقليدية</h2>

<p>أدوات الفحص التقليدية مثل ESLint و Pylint و PHPStan ممتازة فيما تفعله. فهي تكشف أخطاء بناء الجملة، وتفرض اتفاقيات الأسلوب، وتُعلّم المتغيرات غير المستخدمة، وتضمن تنسيقاً متسقاً. لكنها تعمل ضمن قيد أساسي: تحلل الملفات بمعزل عن بعضها.</p>

<p>تأمل تطبيق Django كبير يحتوي على 400 وحدة Python. يمكن لـ Pylint إخبارك أن استيراداً معيناً غير مستخدم <em>داخل ملف واحد</em>. لكنه لا يستطيع إخبارك أن الوحدة A تعتمد على الوحدة B، التي تعتمد على الوحدة C، التي تعتمد بدورها على الوحدة A، مما يُنشئ تبعية دائرية تجعل اختبار النظام الفرعي بأكمله بمعزل أمراً مستحيلاً. لا يستطيع إخبارك أن فئة مساعدة في <code>utils/helpers.py</code> يتم استيرادها بواسطة 47 ملفاً مختلفاً، مما يجعلها عنق زجاجة خطيراً حيث ينتشر تغيير واحد بشكل لا يمكن التنبؤ به. لا يستطيع إخبارك أن مجلداً كاملاً من فئات المعالجات أصبح يتيماً منذ آخر إعادة هيكلة، يجلس هناك يجمع الغبار ويربك كل مطور جديد يصادفه.</p>

<p>هذه هي <strong>المشكلات المعمارية</strong>. إنها موجودة في العلاقات بين الملفات، وليس داخلها. وهي المشكلات التي تكلف فرق الهندسة أكبر قدر من الوقت، وتسبب أكبر عدد من حوادث الإنتاج، وتخلق أكبر قدر من الإحباط أثناء تأهيل الأعضاء الجدد.</p>

<h2 id="what-ai-brings-to-static-analysis">ما يجلبه الذكاء الاصطناعي للتحليل الثابت</h2>

<p>يعمل التحليل الثابت المدعوم بالذكاء الاصطناعي على مستوى مختلف من التجريد. بدلاً من فحص الملفات الفردية، يبني <strong>رسماً بيانياً للتبعيات</strong> لقاعدة الشيفرة بأكملها. يفهم أي الوحدات تستورد أياً، وكيف تتدفق الرموز بين الملفات، وأين توجد نقاط الضغط الهيكلية.</p>

<p>هنا يختلف AigisCode عن المقاربات التقليدية. يشغّل AigisCode خط أنابيب من ست مراحل: Index و Graph و Detect و Rules و AI Review و Report. المراحل الثلاث الأولى حتمية بالكامل. يحلل Tree-sitter الملفات المصدرية عبر PHP و Python و TypeScript و JavaScript و Vue. يبني petgraph رسماً بيانياً كاملاً للتبعيات. تحدد الكواشف المرشحين للتبعيات الدائرية والشيفرة الميتة والقيم المضمنة والانتهاكات المعمارية.</p>

<p>يدخل الذكاء الاصطناعي في المرحلة الخامسة. بعد أن تنتج الكواشف الحتمية مرشحين بمستويات ثقة، تصنف مرحلة مراجعة الذكاء الاصطناعي النتائج كإيجابيات حقيقية أو إيجابيات كاذبة أو تحتاج سياقاً. هذا النهج الهجين بالغ الأهمية. تضمن المراحل الحتمية قابلية إعادة الإنتاج والشرح. تتعامل مرحلة الذكاء الاصطناعي مع الفروق الدقيقة التي لا تستطيع الأساليب الاستدلالية البحتة التقاطها.</p>

<h2 id="the-rise-of-ai-coding-agents">صعود وكلاء البرمجة بالذكاء الاصطناعي</h2>

<p>ربما كان الاتجاه الأبرز في عامي 2025 و 2026 هو انتشار وكلاء البرمجة بالذكاء الاصطناعي. يمكن لأدوات مثل Claude Code و GitHub Copilot Workspace و وكلاء Codex الآن تنفيذ الميزات وإصلاح الأخطاء وإعادة هيكلة الشيفرة بشكل مستقل. لكن هؤلاء الوكلاء يحتاجون إلى تغذية راجعة منظمة وقابلة للقراءة آلياً حول صحة قاعدة الشيفرة للعمل بفعالية.</p>

<p>مخرجات أداة الفحص مفيدة لوكيل الذكاء الاصطناعي، لكنها محدودة. يمكن للوكيل إصلاح تحذير استيراد غير مستخدم. لكن هل يستطيع فهم أن إصلاح تبعية دائرية بين ثلاث وحدات يتطلب إعادة هيكلة التجريدات المشتركة؟ هل يستطيع تحديد أي من 23 اكتشافاً للشيفرة الميتة يجب معالجته أولاً بناءً على ملف المخاطر للشيفرة المحيطة؟</p>

<p>هنا تصبح أدوات التحليل المدعومة بالذكاء الاصطناعي النظام الحسي لوكلاء الذكاء الاصطناعي. ينتج AigisCode تقريراً منظماً بصيغة JSON في <code>.aigiscode/deterministic-analysis.json</code> يمكن للوكيل تحليله مباشرة. يتضمن التقرير مستويات الخطورة ودرجات الثقة ومسارات الملفات والتفسيرات. يمكن للوكيل قراءة هذا التقرير وفرز النتائج حسب الثقة والبدء في إصلاح المشكلات الأكثر تأثيراً تلقائياً.</p>

<h2 id="the-numbers-tell-the-story">الأرقام تروي القصة</h2>

<p>تأثير المشكلات المعمارية موثق جيداً. وجدت دراسة أجرتها Stripe في عام 2025 أن المطورين يقضون ما يقدر بـ <strong>42% من وقتهم</strong> في التعامل مع الديون التقنية والصيانة، ارتفاعاً من 33% في عام 2018. التبعيات الدائرية سبب رئيسي لـ "جحيم التبعيات" في المونوليثات الكبيرة، ووجد تحليل عام 2024 لـ 10,000 مشروع Python مفتوح المصدر أن <strong>34% احتوت على تبعية دائرية قوية واحدة على الأقل</strong> أثرت على عزل الاختبارات.</p>

<p>الشيفرة الميتة خبيثة بنفس القدر. قدّر بحث من جامعة زيورخ في عام 2025 أن قاعدة الشيفرة المؤسسية المتوسطة تحتوي على <strong>12-18% شيفرة ميتة</strong> من حيث الحجم. هذه الشيفرة الميتة تزيد أوقات البناء، وتوسع سطح الهجوم الأمني، وتربك المطورين الذين يقرؤون الشيفرة، وتضخم أحجام الحزم لتطبيقات الواجهة الأمامية.</p>

<p>أدوات الفحص التقليدية لا تكشف أياً من هذا. الملف الذي يحتوي على شيفرة ميتة صحيح نحوياً. التبعية الدائرية تتضمن ملفات صحيحة فردياً. لا تصبح المشكلات مرئية إلا عندما تنظر إلى قاعدة الشيفرة ككل.</p>

<h2 id="how-aigiscode-fits-the-ecosystem">كيف يتناسب AigisCode مع المنظومة</h2>

<p>AigisCode لا يحل محل أداة الفحص الخاصة بك. إنه يكملها. فكر في أدوات الفحص كمدقق إملائي و AigisCode كمحررك الهيكلي. يضمن ESLint أن JavaScript الخاص بك يتبع أنماطاً متسقة. يضمن AigisCode أن وحداتك لا تشكل دورات تبعية تجعل نشر تطبيقك تدريجياً أمراً مستحيلاً.</p>

<p>الأداة مفتوحة المصدر بموجب رخصة MIT. أمر واحد، <code>aigiscode analyze /path/to/project</code>، يشغل خط الأنابيب الكامل من ست مراحل. المخرجات هي تقرير JSON قابل للقراءة آلياً يمكن للبشر ووكلاء الذكاء الاصطناعي استهلاكه. تتيح ملفات السياسة للفرق تخصيص سلوك الكشف لكل مشروع، وكبت الإيجابيات الكاذبة وترميز المعرفة الخاصة بالمشروع.</p>

<p>تتعامل ملفات تعريف الإضافات المدمجة لـ Django و Laravel و WordPress مع الأنماط الخاصة بإطار العمل دون أي تكوين إضافي. على سبيل المثال، يتعرف إضافة Django على أن فئات النماذج المشار إليها في عمليات الترحيل ليست شيفرة ميتة، حتى لو لم يستوردها أي ملف Python مباشرة. تفهم إضافة Laravel ربط مزودي الخدمة ومحددات الواجهة. هذه الأنواع من التعديلات الواعية بإطار العمل تزيل فئات كاملة من الإيجابيات الكاذبة دون الحاجة إلى تكوين يدوي.</p>

<h2 id="looking-ahead">نظرة نحو المستقبل</h2>

<p>المسار واضح. مع نمو قواعد الشيفرة وتولي وكلاء الذكاء الاصطناعي مزيداً من أعمال التطوير، ستزداد الحاجة إلى تحليل هيكلي يتجاوز فحص مستوى الملف. نحن نتجه نحو عالم يتم فيه تقييم كل طلب سحب ليس فقط من حيث أسلوب الشيفرة وتغطية الاختبارات، بل من حيث تأثيره على رسم التبعيات البياني ومساهمته في الديون التقنية وتوافقه مع البنية المعمارية المقصودة.</p>

<p>تحليل الشيفرة المدعوم بالذكاء الاصطناعي هو الجسر بين أدوات الفحص اليوم وأنظمة جودة الشيفرة المؤتمتة بالكامل في الغد. الفرق التي تتبناه الآن ستتمتع ببنى معمارية أنظف وتأهيل أسرع ومفاجآت إنتاج أقل. الأدوات موجودة. الأنماط مُثبتة. السؤال لم يعد هل يهم تحليل الشيفرة بالذكاء الاصطناعي. بل هل يمكن لفريقك تحمل العمل بدونه.</p>
`,
      pl: `<h2 id="why-ai">Dlaczego analiza kodu AI ma znaczenie</h2>
<p>Tradycyjne lintery sprawdzają pliki pojedynczo. AigisCode analizuje cały graf zależności, wykrywając problemy architektoniczne niewidoczne na poziomie pliku.</p>`,
      bn: `
<p>২০২৬ সাল কোড কোয়ালিটি সম্পর্কে আমাদের চিন্তাভাবনায় একটি মৌলিক পরিবর্তন এনেছে। আমরা সেই যুগ পার করে এসেছি যখন একটি লিন্টার চালানোকেই যথেষ্ট পরিশ্রম বলে মনে করা হতো। আজ, কোডবেসগুলো লক্ষ লক্ষ ফাইল জুড়ে বিস্তৃত, মাইক্রোসার্ভিসগুলো অদৃশ্য কন্ট্র্যাক্টের মাধ্যমে যোগাযোগ করে, এবং AI কোডিং এজেন্টরা এমন গতিতে কোড তৈরি করে যা শুধুমাত্র মানুষের রিভিউ দিয়ে মেলানো সম্ভব নয়। এই পরিস্থিতিতে, AI-চালিত কোড বিশ্লেষণ কোনো বিলাসিতা নয়। এটি একটি প্রয়োজনীয়তা।</p>

<h2 id="the-limitations-of-traditional-linters">প্রচলিত লিন্টারের সীমাবদ্ধতা</h2>

<p>ESLint, Pylint, এবং PHPStan-এর মতো প্রচলিত লিন্টারগুলো তাদের কাজে চমৎকার। তারা সিনট্যাক্স ত্রুটি ধরে, স্টাইল কনভেনশন প্রয়োগ করে, অব্যবহৃত ভেরিয়েবল চিহ্নিত করে এবং সামঞ্জস্যপূর্ণ ফরম্যাটিং নিশ্চিত করে। কিন্তু তারা একটি মৌলিক সীমাবদ্ধতার মধ্যে কাজ করে: তারা ফাইলগুলো আলাদাভাবে বিশ্লেষণ করে।</p>

<p>৪০০টি Python মডিউল সহ একটি বড় Django অ্যাপ্লিকেশন বিবেচনা করুন। Pylint আপনাকে বলতে পারে যে একটি নির্দিষ্ট ইমপোর্ট <em>একটি একক ফাইলের মধ্যে</em> অব্যবহৃত। এটি আপনাকে বলতে পারে না যে মডিউল A মডিউল B-এর উপর নির্ভর করে, যা মডিউল C-এর উপর নির্ভর করে, যা আবার মডিউল A-এর উপর নির্ভর করে, একটি সার্কুলার ডিপেন্ডেন্সি তৈরি করে যা সম্পূর্ণ সাবসিস্টেমকে আলাদাভাবে টেস্ট করা অসম্ভব করে তোলে। এটি আপনাকে বলতে পারে না যে <code>utils/helpers.py</code>-তে একটি ইউটিলিটি ক্লাস ৪৭টি ভিন্ন ফাইল দ্বারা ইমপোর্ট করা হয়, এটিকে একটি বিপজ্জনক বটলনেক করে তুলছে যেখানে একটি একক পরিবর্তন অনির্দেশ্যভাবে ছড়িয়ে পড়ে। এটি আপনাকে বলতে পারে না যে হ্যান্ডলার ক্লাসের একটি সম্পূর্ণ ডিরেক্টরি শেষ রিফ্যাক্টরিংয়ের পর থেকে অনাথ হয়ে পড়ে আছে, সেখানে ধুলো জমাচ্ছে এবং প্রতিটি নতুন ডেভেলপারকে বিভ্রান্ত করছে।</p>

<p>এগুলো হলো <strong>আর্কিটেকচারাল সমস্যা</strong>। এগুলো ফাইলগুলোর মধ্যকার সম্পর্কে বিদ্যমান, তাদের ভিতরে নয়। এবং এগুলোই সেই সমস্যা যা ইঞ্জিনিয়ারিং টিমের সবচেয়ে বেশি সময় নেয়, সবচেয়ে বেশি প্রোডাকশন ইনসিডেন্ট ঘটায় এবং অনবোর্ডিংয়ের সময় সবচেয়ে বেশি হতাশা তৈরি করে।</p>

<h2 id="what-ai-brings-to-static-analysis">AI স্ট্যাটিক অ্যানালিসিসে কী নিয়ে আসে</h2>

<p>AI-চালিত স্ট্যাটিক অ্যানালিসিস ভিন্ন স্তরের অ্যাবস্ট্রাকশনে কাজ করে। পৃথক ফাইল পরীক্ষা করার পরিবর্তে, এটি আপনার সম্পূর্ণ কোডবেসের একটি <strong>ডিপেন্ডেন্সি গ্রাফ</strong> তৈরি করে। এটি বোঝে কোন মডিউলগুলো কোনটি ইমপোর্ট করে, কিভাবে সিম্বলগুলো ফাইলের মধ্যে প্রবাহিত হয় এবং স্ট্রাকচারাল প্রেশার পয়েন্টগুলো কোথায়।</p>

<p>এখানেই AigisCode-এর মতো টুলগুলো প্রচলিত পদ্ধতি থেকে আলাদা। AigisCode একটি ছয়-পর্যায়ের পাইপলাইন চালায়: Index, Graph, Detect, Rules, AI Review, এবং Report। প্রথম তিনটি পর্যায় সম্পূর্ণ ডিটারমিনিস্টিক। Tree-sitter PHP, Python, TypeScript, JavaScript, এবং Vue জুড়ে সোর্স ফাইল পার্স করে। petgraph একটি সম্পূর্ণ ডিপেন্ডেন্সি গ্রাফ তৈরি করে। ডিটেক্টরগুলো সার্কুলার ডিপেন্ডেন্সি, ডেড কোড, হার্ডওয়্যার্ড ভ্যালু এবং আর্কিটেকচারাল লঙ্ঘনের প্রার্থী চিহ্নিত করে।</p>

<p>AI পঞ্চম পর্যায়ে প্রবেশ করে। ডিটারমিনিস্টিক ডিটেক্টরগুলো কনফিডেন্স লেভেল সহ প্রার্থী তৈরি করার পর, AI রিভিউ পর্যায় ফলাফলগুলোকে true positive, false positive, বা needs-context হিসেবে শ্রেণীবদ্ধ করে। এই হাইব্রিড পদ্ধতি গুরুত্বপূর্ণ। ডিটারমিনিস্টিক পর্যায়গুলো পুনরুৎপাদনযোগ্যতা এবং ব্যাখ্যাযোগ্যতা নিশ্চিত করে। AI পর্যায় সেই সূক্ষ্মতা সামলায় যা বিশুদ্ধ হিউরিস্টিকস ধরতে পারে না।</p>

<h2 id="the-rise-of-ai-coding-agents">AI কোডিং এজেন্টের উত্থান</h2>

<p>সম্ভবত ২০২৫ এবং ২০২৬-এর সবচেয়ে উল্লেখযোগ্য প্রবণতা হলো AI কোডিং এজেন্টের বিস্তার। Claude Code, GitHub Copilot Workspace, এবং Codex এজেন্টের মতো টুলগুলো এখন স্বায়ত্তশাসিতভাবে ফিচার ইমপ্লিমেন্ট করতে, বাগ ফিক্স করতে এবং কোড রিফ্যাক্টর করতে পারে। কিন্তু এই এজেন্টদের কার্যকরভাবে কাজ করার জন্য কোডবেস স্বাস্থ্য সম্পর্কে স্ট্রাকচার্ড, মেশিন-রিডেবল ফিডব্যাক প্রয়োজন।</p>

<p>একটি লিন্টারের আউটপুট একটি AI এজেন্টের জন্য দরকারী, কিন্তু সীমিত। একটি এজেন্ট একটি অব্যবহৃত ইমপোর্ট সতর্কতা ঠিক করতে পারে। কিন্তু এটি কি বুঝতে পারে যে তিনটি মডিউলের মধ্যে একটি সার্কুলার ডিপেন্ডেন্সি ঠিক করতে শেয়ার্ড অ্যাবস্ট্রাকশনগুলো পুনর্গঠন করা প্রয়োজন? এটি কি ২৩টি ডেড কোড ফলাফলের মধ্যে কোনটি আগে সমাধান করতে হবে তা ঠিক করতে পারে আশেপাশের কোডের ঝুঁকি প্রোফাইলের উপর ভিত্তি করে?</p>

<p>এখানেই AI-চালিত বিশ্লেষণ টুলগুলো AI এজেন্টের সেন্সরি সিস্টেম হয়ে ওঠে। AigisCode <code>.aigiscode/deterministic-analysis.json</code>-এ একটি স্ট্রাকচার্ড JSON রিপোর্ট আউটপুট করে যা একটি এজেন্ট সরাসরি পার্স করতে পারে। রিপোর্টে সেভারিটি লেভেল, কনফিডেন্স স্কোর, ফাইল পাথ এবং ব্যাখ্যা অন্তর্ভুক্ত থাকে। একটি এজেন্ট এই রিপোর্ট পড়তে পারে, কনফিডেন্স অনুযায়ী ফলাফল ট্রায়াজ করতে পারে এবং স্বয়ংক্রিয়ভাবে সবচেয়ে প্রভাবশালী সমস্যাগুলো ঠিক করা শুরু করতে পারে।</p>

<h2 id="the-numbers-tell-the-story">সংখ্যাই গল্প বলে</h2>

<p>আর্কিটেকচারাল সমস্যাগুলোর প্রভাব সুপ্রতিষ্ঠিত। Stripe-এর ২০২৫ সালের একটি গবেষণায় দেখা গেছে যে ডেভেলপাররা তাদের সময়ের আনুমানিক <strong>৪২%</strong> টেকনিক্যাল ডেট এবং রক্ষণাবেক্ষণে ব্যয় করে, যা ২০১৮ সালে ৩৩% ছিল। সার্কুলার ডিপেন্ডেন্সি বড় মনোলিথে "dependency hell"-এর একটি প্রধান কারণ, এবং ১০,০০০টি ওপেন-সোর্স Python প্রজেক্টের ২০২৪ সালের একটি বিশ্লেষণে দেখা গেছে যে <strong>৩৪% অন্তত একটি শক্তিশালী সার্কুলার ডিপেন্ডেন্সি ধারণ করে</strong> যা টেস্ট আইসোলেশনকে প্রভাবিত করেছে।</p>

<p>ডেড কোডও সমানভাবে ক্ষতিকর। University of Zurich-এর ২০২৫ সালের গবেষণায় অনুমান করা হয়েছে যে গড় এন্টারপ্রাইজ কোডবেসে ভলিউম অনুযায়ী <strong>১২-১৮% ডেড কোড</strong> থাকে। এই ডেড কোড বিল্ড টাইম বাড়ায়, সিকিউরিটি অ্যাটাক সারফেস বিস্তৃত করে, কোড পড়া ডেভেলপারদের বিভ্রান্ত করে এবং ফ্রন্টএন্ড অ্যাপ্লিকেশনের বান্ডেল সাইজ ফুলিয়ে তোলে।</p>

<p>প্রচলিত লিন্টারগুলো এর কোনোটিই ধরতে পারে না। ডেড কোড সম্বলিত একটি ফাইল সিনট্যাক্টিক্যালি বৈধ। একটি সার্কুলার ডিপেন্ডেন্সিতে এমন ফাইল জড়িত যা পৃথকভাবে সঠিক। সমস্যাগুলো তখনই দৃশ্যমান হয় যখন আপনি কোডবেসকে সামগ্রিকভাবে দেখেন।</p>

<h2 id="how-aigiscode-fits-the-ecosystem">AigisCode কিভাবে ইকোসিস্টেমে মানানসই হয়</h2>

<p>AigisCode আপনার লিন্টার প্রতিস্থাপন করে না। এটি সম্পূরক। লিন্টারকে আপনার বানান-পরীক্ষক এবং AigisCode-কে আপনার স্ট্রাকচারাল এডিটর হিসেবে ভাবুন। ESLint নিশ্চিত করে আপনার JavaScript সামঞ্জস্যপূর্ণ প্যাটার্ন অনুসরণ করে। AigisCode নিশ্চিত করে আপনার মডিউলগুলো ডিপেন্ডেন্সি সাইকেল তৈরি করে না যা আপনার অ্যাপ্লিকেশনকে ক্রমবর্ধমানভাবে ডিপ্লয় করা অসম্ভব করে তোলে।</p>

<p>টুলটি MIT লাইসেন্সের অধীনে ওপেন সোর্স এবং pip-এর মাধ্যমে ইনস্টলযোগ্য। একটি একক কমান্ড, <code>aigiscode analyze /path/to/project</code>, সম্পূর্ণ ছয়-পর্যায়ের পাইপলাইন চালায়। আউটপুট হলো একটি মেশিন-রিডেবল JSON রিপোর্ট যা মানুষ এবং AI এজেন্ট উভয়ই ব্যবহার করতে পারে। পলিসি ফাইলগুলো টিমদের প্রতি প্রজেক্টে ডিটেকশন আচরণ কাস্টমাইজ করতে, false positive দমন করতে এবং প্রজেক্ট-নির্দিষ্ট জ্ঞান এনকোড করতে দেয়।</p>

<p>Django, Laravel, এবং WordPress-এর জন্য বিল্ট-ইন প্লাগইন প্রোফাইলগুলো ফ্রেমওয়ার্ক-নির্দিষ্ট প্যাটার্ন বক্সের বাইরে সামলায়। উদাহরণস্বরূপ, Django প্লাগইন স্বীকৃতি দেয় যে মাইগ্রেশনে রেফারেন্সকৃত মডেল ক্লাসগুলো ডেড কোড নয়, এমনকি কোনো Python ফাইল সরাসরি তাদের ইমপোর্ট না করলেও। Laravel প্লাগইন সার্ভিস প্রোভাইডার বাইন্ডিং এবং ফ্যাসাড অ্যাক্সেসর বোঝে। এই ধরনের ফ্রেমওয়ার্ক-সচেতন সমন্বয় ম্যানুয়াল কনফিগারেশন ছাড়াই false positive-এর সম্পূর্ণ বিভাগগুলো দূর করে।</p>

<h2 id="looking-ahead">সামনের দিকে তাকানো</h2>

<p>গতিপথ স্পষ্ট। কোডবেসগুলো যত বড় হচ্ছে এবং AI এজেন্টরা যত বেশি উন্নয়নের কাজ নিচ্ছে, ফাইল-লেভেল লিন্টিংয়ের বাইরে যাওয়া স্ট্রাকচারাল বিশ্লেষণের প্রয়োজনীয়তা কেবল বাড়বে। আমরা এমন একটি বিশ্বের দিকে এগিয়ে যাচ্ছি যেখানে প্রতিটি পুল রিকোয়েস্ট শুধু কোড স্টাইল এবং টেস্ট কভারেজের জন্যই নয়, বরং ডিপেন্ডেন্সি গ্রাফে এর প্রভাব, টেকনিক্যাল ডেটে এর অবদান এবং উদ্দিষ্ট আর্কিটেকচারের সাথে এর সামঞ্জস্যের জন্যও মূল্যায়ন করা হবে।</p>

<p>AI-চালিত কোড বিশ্লেষণ হলো আজকের লিন্টার এবং আগামীকালের সম্পূর্ণ স্বয়ংক্রিয় কোড কোয়ালিটি সিস্টেমের মধ্যে সেতু। যে টিমগুলো এখন এটি গ্রহণ করবে তাদের পরিষ্কার আর্কিটেকচার, দ্রুত অনবোর্ডিং এবং কম প্রোডাকশন বিস্ময় থাকবে। টুলগুলো বিদ্যমান। প্যাটার্নগুলো প্রমাণিত। প্রশ্ন আর এটা নয় যে AI কোড বিশ্লেষণ গুরুত্বপূর্ণ কিনা। প্রশ্ন হলো আপনার টিম এটি ছাড়া কাজ করার সামর্থ্য রাখে কিনা।</p>
`,
    },
  },

  /* ======================================================================== */
  /*  2. The Real Cost of Circular Dependencies                               */
  /* ======================================================================== */
  {
    slug: 'circular-dependencies-real-cost',
    date: '2026-02-10',
    readTime: 10,
    tags: ['Architecture', 'Circular Dependencies', 'Refactoring'],
    image: '/blog-circular-dependencies.jpg',
    author: { name: 'David Strejc', role: 'Creator of AigisCode' },
    relatedSlugs: [
      'dead-code-technical-debt',
      'static-analysis-vs-linters-2026',
    ],
    title: {
      en: 'The Real Cost of Circular Dependencies in Your Codebase',
      cs: 'Skutečná cena cyklických závislostí ve vašem kódu',
      fr: 'Le vrai cout des dependances circulaires dans votre codebase',
      es: 'El costo real de las dependencias circulares en tu codigo',
      zh: '代码库中循环依赖的真实代价',
      hi: 'आपके कोडबेस में सर्कुलर डिपेंडेंसी की वास्तविक लागत',
      pt: 'O custo real das dependências circulares no seu código',
      ar: 'التكلفة الحقيقية للتبعيات الدائرية في قاعدة شيفرتك',
      pl: 'Rzeczywisty koszt cyklicznych zależności w Twojej bazie kodu',
      bn: 'আপনার কোডবেসে সার্কুলার ডিপেন্ডেন্সির প্রকৃত মূল্য',
    },
    description: {
      en: 'Circular dependencies silently erode your codebase. Learn what they are, why they are dangerous, and how to detect and fix them before they become architectural nightmares.',
      cs: 'Cyklické závislosti tiše narušují kód. Zjistěte, jak je detekovat a opravit.',
      fr: 'Les dependances circulaires erodent silencieusement votre codebase. Apprenez a les detecter et les corriger.',
      es: 'Las dependencias circulares erosionan silenciosamente tu codigo. Aprende a detectarlas y corregirlas.',
      zh: '循环依赖默默侵蚀你的代码库。了解如何检测和修复它们。',
      hi: 'सर्कुलर डिपेंडेंसी चुपचाप आपके कोडबेस को नष्ट करती हैं। उन्हें कैसे पहचानें और ठीक करें।',
      pt: 'Dependências circulares corroem silenciosamente seu código. Aprenda a detectá-las e corrigi-las.',
      ar: 'تآكل التبعيات الدائرية قاعدة شيفرتك بصمت. تعرّف ما هي ولماذا هي خطيرة وكيف تكتشفها وتصلحها قبل أن تصبح كوابيس معمارية.',
      pl: 'Cykliczne zależności po cichu niszczą Twoją bazę kodu. Dowiedz się, czym są, dlaczego są niebezpieczne i jak je wykrywać, zanim staną się niekontrolowanym długiem technicznym.',
      bn: 'সার্কুলার ডিপেন্ডেন্সি নীরবে আপনার কোডবেস ক্ষয় করে। জানুন এগুলো কী, কেন বিপজ্জনক এবং আর্কিটেকচারাল দুঃস্বপ্ন হওয়ার আগে কিভাবে শনাক্ত ও সমাধান করবেন।',
    },
    metaDescription: {
      en: 'Understand the real cost of circular dependencies: compilation failures, testing nightmares, and deployment coupling. Learn detection strategies and practical fixes with AigisCode.',
      cs: 'Pochopte skutečnou cenu cyklických závislostí a naučte se je detekovat a opravovat s AigisCode.',
      fr: 'Comprenez le vrai cout des dependances circulaires et apprenez a les detecter avec AigisCode.',
      es: 'Entienda el costo real de las dependencias circulares y aprenda a detectarlas con AigisCode.',
      zh: '了解循环依赖的真实代价以及如何使用 AigisCode 检测和修复它们。',
      hi: 'सर्कुलर डिपेंडेंसी की वास्तविक लागत और AigisCode से उन्हें कैसे पहचानें।',
      pt: 'Entenda o custo real das dependências circulares e aprenda a detectá-las com o AigisCode.',
      ar: 'افهم التكلفة الحقيقية للتبعيات الدائرية: فشل التجميع وكوابيس الاختبار واقتران النشر. تعلّم استراتيجيات الاكتشاف والإصلاحات العملية مع AigisCode.',
      pl: 'Poznaj rzeczywisty koszt cyklicznych zależności: błędy kompilacji, problemy z testowaniem i wąskie gardła wdrożeń. Dowiedz się, jak AigisCode je wykrywa za pomocą analizy Tarjan SCC.',
      bn: 'সার্কুলার ডিপেন্ডেন্সির প্রকৃত মূল্য বুঝুন: কম্পাইলেশন ব্যর্থতা, টেস্টিং দুঃস্বপ্ন এবং ডিপ্লয়মেন্ট কাপলিং। AigisCode দিয়ে শনাক্তকরণ কৌশল ও ব্যবহারিক সমাধান শিখুন।',
    },
    content: {
      en: `
<p>Every codebase has a shape. You might not see it in your editor, but it is there: a web of dependencies connecting modules, classes, and functions. When that web contains loops, where module A depends on module B, and module B depends back on module A, you have a circular dependency. And while it may seem like a minor structural quirk, circular dependencies are one of the most expensive forms of technical debt a codebase can accumulate.</p>

<h2 id="what-circular-dependencies-are">What Circular Dependencies Actually Are</h2>

<p>A circular dependency exists when two or more modules form a dependency cycle. The simplest case is a direct cycle: <code>auth.py</code> imports from <code>users.py</code>, and <code>users.py</code> imports from <code>auth.py</code>. But real-world cycles are often longer and harder to spot. A cycle might involve four or five modules, each individually looking clean, but together forming a loop that ties them into an inseparable unit.</p>

<p>It is important to distinguish between two types of cycles. <strong>Strong circular dependencies</strong> are architectural cycles. They exist at the module or package level and indicate that two subsystems are fundamentally entangled. <strong>Total circular dependencies</strong> include runtime and load-order cycles that may exist due to lazy imports, conditional requires, or framework magic. Both are worth knowing about, but strong cycles are the ones that cause the most damage.</p>

<p>AigisCode makes this distinction explicit in its analysis. The JSON report separates <code>strong_circular_dependencies</code> from <code>circular_dependencies</code>, allowing teams to prioritize the architectural cycles that genuinely need refactoring while noting the runtime cycles for awareness.</p>

<h2 id="why-they-are-dangerous">Why Circular Dependencies Are Dangerous</h2>

<h3>Compilation and Load-Order Failures</h3>

<p>In languages with strict module resolution, circular dependencies can cause outright failures. Python's import system will partially execute a module when it encounters a cycle, leading to <code>ImportError</code> or <code>AttributeError</code> at runtime when a name has not yet been defined. In TypeScript with strict ES module semantics, circular imports can result in <code>undefined</code> values at the point of use because the module has not finished initializing. In PHP with autoloading, circular dependencies can cause subtle bugs where a class appears to be available but its dependencies have not been loaded yet.</p>

<p>These failures are notoriously difficult to debug because they depend on <em>import order</em>, which varies based on which entry point triggered the code path. The same test suite might pass or fail depending on which test runs first.</p>

<h3>Testing Nightmares</h3>

<p>Circular dependencies make unit testing extraordinarily difficult. If module A depends on module B and module B depends on module A, you cannot test either in isolation without mocking the other. This creates a situation where your test setup is more complex than the code being tested, and your mocks might not accurately reflect the real behavior of the dependency.</p>

<p>Consider a real example from a Django application. The <code>orders</code> module imports from <code>inventory</code> to check stock levels. The <code>inventory</code> module imports from <code>orders</code> to calculate reserved quantities. Now, to unit test the <code>orders</code> module, you need to mock the inventory check. But the mock needs to understand reserved quantities, which requires understanding orders. You end up with circular mock dependencies that mirror the circular code dependencies, and your tests become brittle, slow, and unreliable.</p>

<h3>Deployment Coupling</h3>

<p>In a microservice or modular monolith architecture, circular dependencies prevent independent deployment. If service A depends on service B and service B depends on service A, you cannot deploy either one independently. Every change to either service requires coordinated deployment of both, eliminating one of the primary benefits of modular architecture.</p>

<p>This coupling extends to team boundaries. If team Alpha owns module A and team Beta owns module B, a circular dependency means neither team can ship without coordinating with the other. Velocity drops. Sprint planning becomes a negotiation. And the pressure to "just add one more import" grows because the modules are already coupled anyway.</p>

<h3>Refactoring Paralysis</h3>

<p>Perhaps the most insidious cost is that circular dependencies make refactoring feel impossible. When modules are tightly coupled in a cycle, changing the interface of one module requires changing all the others in the cycle simultaneously. There is no way to do it incrementally. This leads to a "big bang" refactoring mindset, where teams postpone structural improvements because the scope feels overwhelming, and the cycle gets worse with every sprint.</p>

<h2 id="real-world-examples">Real-World Examples</h2>

<h3>The Django Settings Cycle</h3>

<p>A common pattern in Django projects is a cycle between <code>settings</code>, <code>models</code>, and <code>utils</code>. Settings imports from utils for path resolution. Utils imports from models for database queries. Models imports from settings for configuration values. This three-way cycle means that changing the settings structure requires touching the utility layer and potentially the model layer, and vice versa.</p>

<h3>The Node.js Controller-Service Loop</h3>

<p>In Express.js applications, it is common to see controllers importing services and services importing controllers (often for error handling or response formatting). The fix is straightforward: introduce a shared error/response module that both layers depend on, breaking the cycle. But without a tool to detect the cycle, teams often do not realize it exists until they try to extract the service layer into a shared library and discover it cannot stand alone.</p>

<h3>The Laravel Event-Listener Tangle</h3>

<p>Laravel applications frequently develop cycles between event classes and their listeners. An event in the <code>Orders</code> namespace triggers a listener in <code>Inventory</code>, which dispatches an event back to <code>Orders</code>. Individually, each class is clean. Together, they form a runtime cycle that can cause infinite loops under specific conditions and makes the event flow impossible to reason about without a graph visualization.</p>

<h2 id="how-to-detect-and-fix">How to Detect and Fix Circular Dependencies</h2>

<h3>Detection with AigisCode</h3>

<p>The first step is visibility. Run <code>aigiscode analyze /path/to/project</code> and examine the <code>graph_analysis.strong_circular_dependencies</code> field in the JSON report. Each entry lists the modules involved in the cycle and the import paths that create it. This gives you a precise map of where the cycles are and which imports need to be restructured.</p>

<h3>The Dependency Inversion Fix</h3>

<p>The most common fix for circular dependencies is <strong>dependency inversion</strong>. Instead of module A importing directly from module B and vice versa, you introduce an interface or abstract base class that both modules depend on. Module A depends on the interface. Module B implements the interface. The dependency arrow now points in one direction only.</p>

<p>In Python, this often means creating a <code>protocols.py</code> or <code>interfaces.py</code> module that defines the contracts between subsystems. In TypeScript, it means extracting shared types into a <code>types/</code> directory that both modules import from without importing from each other.</p>

<h3>The Mediator Pattern</h3>

<p>For event-driven cycles, the mediator pattern is effective. Instead of modules communicating directly, they communicate through a shared event bus or mediator. Module A dispatches an event. Module B listens for it. Neither imports from the other. The mediator is the only shared dependency, and it contains no business logic, just routing.</p>

<h3>The Extract-and-Share Pattern</h3>

<p>Sometimes the cycle exists because two modules share a concept that has not been given its own home. The fix is to extract the shared concept into a new module that both original modules depend on. For example, if <code>orders.py</code> and <code>inventory.py</code> both need a <code>ReservationCalculator</code>, extract it into <code>reservations.py</code> and let both modules import from it.</p>

<h2 id="prevention-is-cheaper-than-cure">Prevention Is Cheaper Than Cure</h2>

<p>The best approach is to detect circular dependencies early and prevent new ones from forming. Integrate AigisCode into your CI pipeline. Run it on every pull request. If a new strong circular dependency appears, the PR should be flagged for architectural review before merging.</p>

<p>The cost of fixing a circular dependency grows exponentially with the age of the cycle. A cycle caught in a PR takes minutes to fix. A cycle that has been growing for two years might take weeks of coordinated refactoring. The tools to catch them early exist today. The only question is whether your team uses them.</p>
`,
      cs: `
<p>Každý codebase má svůj tvar. V editoru ho nevidíte, ale je tam: síť závislostí propojující moduly, třídy a funkce. Když tato síť obsahuje smyčky — kde modul A závisí na modulu B a modul B závisí zpět na modulu A — máte cyklickou závislost. A i když se to může zdát jako drobná strukturální zvláštnost, cyklické závislosti jsou jednou z nejdražších forem technického dluhu, které se v codebase mohou nahromadit.</p>

<h2 id="what-circular-dependencies-are">Co jsou cyklické závislosti</h2>

<p>Cyklická závislost existuje, když dva nebo více modulů tvoří cyklus závislostí. Nejjednodušší případ je přímý cyklus: <code>auth.py</code> importuje z <code>users.py</code> a <code>users.py</code> importuje z <code>auth.py</code>. Ale reálné cykly jsou často delší a hůře odhalitelné. Cyklus může zahrnovat čtyři nebo pět modulů, z nichž každý jednotlivě vypadá čistě, ale dohromady tvoří smyčku, která je svazuje do neoddělitelné jednotky.</p>

<p>Je důležité rozlišovat mezi dvěma typy cyklů. <strong>Silné cyklické závislosti</strong> jsou architektonické cykly. Existují na úrovni modulů nebo balíčků a naznačují, že dva subsystémy jsou zásadně propletené. <strong>Celkové cyklické závislosti</strong> zahrnují runtime a load-order cykly, které mohou existovat kvůli lazy importům, podmíněným requires nebo frameworkové magii. O obou stojí za to vědět, ale silné cykly jsou ty, které způsobují největší škody.</p>

<p>AigisCode toto rozlišení ve své analýze explicitně uvádí. JSON report odděluje <code>strong_circular_dependencies</code> od <code>circular_dependencies</code>, což umožňuje týmům upřednostnit architektonické cykly, které skutečně potřebují refaktoring, a zároveň zaznamenat runtime cykly pro informaci.</p>

<h2 id="why-they-are-dangerous">Proč jsou cyklické závislosti nebezpečné</h2>

<h3>Selhání kompilace a pořadí načítání</h3>

<p>V jazycích s přísnou resolucí modulů mohou cyklické závislosti způsobit přímá selhání. Importní systém Pythonu částečně vykoná modul, když narazí na cyklus, což vede k <code>ImportError</code> nebo <code>AttributeError</code> za běhu, když jméno ještě nebylo definováno. V TypeScriptu se striktní sémantikou ES modulů mohou cyklické importy vést k hodnotám <code>undefined</code> v místě použití, protože modul ještě nedokončil inicializaci. V PHP s autoloadingem mohou cyklické závislosti způsobit subtilní chyby, kdy se třída zdá být dostupná, ale její závislosti ještě nebyly načteny.</p>

<p>Tato selhání jsou notoricky obtížně laditelná, protože závisejí na <em>pořadí importů</em>, které se mění podle toho, který vstupní bod spustil danou cestu kódu. Stejná testovací sada může projít nebo selhat v závislosti na tom, který test běží jako první.</p>

<h3>Noční můry testování</h3>

<p>Cyklické závislosti dělají unit testování mimořádně obtížným. Pokud modul A závisí na modulu B a modul B závisí na modulu A, nemůžete testovat ani jeden izolovaně bez mockování toho druhého. To vytváří situaci, kdy je příprava vašeho testu složitější než testovaný kód a vaše mocky nemusí přesně odrážet skutečné chování závislosti.</p>

<p>Zvažte reálný příklad z Django aplikace. Modul <code>orders</code> importuje z <code>inventory</code> pro kontrolu stavu zásob. Modul <code>inventory</code> importuje z <code>orders</code> pro výpočet rezervovaných množství. Nyní, pro unit test modulu <code>orders</code>, potřebujete mockovat kontrolu inventáře. Ale mock potřebuje rozumět rezervovaným množstvím, což vyžaduje pochopení objednávek. Skončíte s cyklickými mock závislostmi, které zrcadlí cyklické závislosti kódu, a vaše testy se stanou křehkými, pomalými a nespolehlivými.</p>

<h3>Provázanost nasazení</h3>

<p>V architektuře mikroslužeb nebo modulárního monolitu cyklické závislosti brání nezávislému nasazení. Pokud služba A závisí na službě B a služba B závisí na službě A, nemůžete nasadit ani jednu nezávisle. Každá změna v kterékoli službě vyžaduje koordinované nasazení obou, čímž se eliminuje jedna z hlavních výhod modulární architektury.</p>

<p>Tato provázanost sahá až k hranicím týmů. Pokud tým Alfa vlastní modul A a tým Beta vlastní modul B, cyklická závislost znamená, že ani jeden tým nemůže dodat bez koordinace s druhým. Rychlost klesá. Plánování sprintů se stává vyjednáváním. A tlak na „přidání ještě jednoho importu" roste, protože moduly jsou už tak provázané.</p>

<h3>Paralýza refaktoringu</h3>

<p>Pravděpodobně nejzákeřnějším nákladem je, že cyklické závislosti dělají refaktoring zdánlivě nemožným. Když jsou moduly těsně provázané v cyklu, změna rozhraní jednoho modulu vyžaduje současnou změnu všech ostatních v cyklu. Nelze to dělat inkrementálně. To vede k mentalitě „velkého třesku" refaktoringu, kdy týmy odkládají strukturální vylepšení, protože rozsah se zdá ohromující, a cyklus se s každým sprintem zhoršuje.</p>

<h2 id="real-world-examples">Příklady z praxe</h2>

<h3>Cyklus Django nastavení</h3>

<p>Běžným vzorem v Django projektech je cyklus mezi <code>settings</code>, <code>models</code> a <code>utils</code>. Settings importuje z utils pro rozlišení cest. Utils importuje z models pro databázové dotazy. Models importuje ze settings pro konfigurační hodnoty. Tento třístranný cyklus znamená, že změna struktury nastavení vyžaduje zásah do utilitní vrstvy a potenciálně do modelové vrstvy, a naopak.</p>

<h3>Smyčka Node.js Controller-Service</h3>

<p>V Express.js aplikacích je běžné vidět kontrolery importující služby a služby importující kontrolery (často pro zpracování chyb nebo formátování odpovědí). Oprava je přímočará: zavést sdílený modul chyb/odpovědí, na kterém obě vrstvy závisejí, čímž se cyklus přeruší. Ale bez nástroje pro detekci cyklu si týmy často neuvědomí, že existuje, dokud se nepokusí extrahovat servisní vrstvu do sdílené knihovny a nezjistí, že nemůže stát samostatně.</p>

<h3>Změť Laravel Event-Listener</h3>

<p>Laravel aplikace často vyvíjejí cykly mezi třídami událostí a jejich listenery. Událost ve jmenném prostoru <code>Orders</code> spustí listener v <code>Inventory</code>, který odešle událost zpět do <code>Orders</code>. Jednotlivě je každá třída čistá. Dohromady tvoří runtime cyklus, který může za specifických podmínek způsobit nekonečné smyčky a znemožňuje uvažování o toku událostí bez vizualizace grafu.</p>

<h2 id="how-to-detect-and-fix">Jak detekovat a opravit cyklické závislosti</h2>

<h3>Detekce s AigisCode</h3>

<p>Prvním krokem je viditelnost. Spusťte <code>aigiscode analyze /path/to/project</code> a prozkoumejte pole <code>graph_analysis.strong_circular_dependencies</code> v JSON reportu. Každý záznam uvádí moduly zapojené do cyklu a importní cesty, které jej vytvářejí. To vám dá přesnou mapu, kde se cykly nacházejí a které importy je třeba restrukturalizovat.</p>

<h3>Oprava inverzí závislostí</h3>

<p>Nejběžnější opravou cyklických závislostí je <strong>inverze závislostí</strong>. Místo toho, aby modul A importoval přímo z modulu B a naopak, zavedete rozhraní nebo abstraktní základní třídu, na které oba moduly závisejí. Modul A závisí na rozhraní. Modul B implementuje rozhraní. Šipka závislosti nyní ukazuje pouze jedním směrem.</p>

<p>V Pythonu to často znamená vytvoření modulu <code>protocols.py</code> nebo <code>interfaces.py</code>, který definuje kontrakty mezi subsystémy. V TypeScriptu to znamená extrakci sdílených typů do adresáře <code>types/</code>, ze kterého oba moduly importují, aniž by importovaly jeden z druhého.</p>

<h3>Vzor Mediátor</h3>

<p>Pro cykly řízené událostmi je efektivní vzor mediátor. Místo přímé komunikace modulů spolu komunikují přes sdílenou sběrnici událostí nebo mediátor. Modul A odešle událost. Modul B ji naslouchá. Ani jeden neimportuje z druhého. Mediátor je jedinou sdílenou závislostí a neobsahuje žádnou business logiku, pouze směrování.</p>

<h3>Vzor extrakce a sdílení</h3>

<p>Někdy cyklus existuje proto, že dva moduly sdílejí koncept, který nedostal svůj vlastní domov. Opravou je extrahovat sdílený koncept do nového modulu, na kterém oba původní moduly závisejí. Například pokud <code>orders.py</code> a <code>inventory.py</code> oba potřebují <code>ReservationCalculator</code>, extrahujte jej do <code>reservations.py</code> a nechte oba moduly z něj importovat.</p>

<h2 id="prevention-is-cheaper-than-cure">Prevence je levnější než léčba</h2>

<p>Nejlepším přístupem je detekovat cyklické závislosti brzy a předcházet vzniku nových. Integrujte AigisCode do vaší CI pipeline. Spouštějte jej na každém pull requestu. Pokud se objeví nová silná cyklická závislost, PR by měl být označen pro architektonickou kontrolu před mergem.</p>

<p>Náklady na opravu cyklické závislosti rostou exponenciálně s věkem cyklu. Cyklus zachycený v PR zabere minuty na opravu. Cyklus, který rostl dva roky, může vyžadovat týdny koordinovaného refaktoringu. Nástroje k jejich včasnému zachycení dnes existují. Jediná otázka je, zda je váš tým používá.</p>
`,
      fr: `
<p>Chaque base de code a une forme. Vous ne la voyez peut-être pas dans votre éditeur, mais elle est là : un réseau de dépendances reliant modules, classes et fonctions. Lorsque ce réseau contient des boucles, où le module A dépend du module B et le module B dépend à son tour du module A, vous avez une dépendance circulaire. Et bien que cela puisse sembler n'être qu'une bizarrerie structurelle mineure, les dépendances circulaires sont l'une des formes les plus coûteuses de dette technique qu'une base de code puisse accumuler.</p>

<h2 id="what-circular-dependencies-are">Ce que sont réellement les dépendances circulaires</h2>

<p>Une dépendance circulaire existe lorsque deux modules ou plus forment un cycle de dépendances. Le cas le plus simple est un cycle direct : <code>auth.py</code> importe depuis <code>users.py</code>, et <code>users.py</code> importe depuis <code>auth.py</code>. Mais les cycles du monde réel sont souvent plus longs et plus difficiles à repérer. Un cycle peut impliquer quatre ou cinq modules, chacun paraissant propre individuellement, mais formant ensemble une boucle qui les lie en une unité inséparable.</p>

<p>Il est important de distinguer deux types de cycles. Les <strong>dépendances circulaires fortes</strong> sont des cycles architecturaux. Elles existent au niveau du module ou du package et indiquent que deux sous-systèmes sont fondamentalement enchevêtrés. Les <strong>dépendances circulaires totales</strong> incluent les cycles d'exécution et d'ordre de chargement qui peuvent exister en raison d'imports paresseux, de requires conditionnels ou de la magie du framework. Les deux méritent d'être connues, mais ce sont les cycles forts qui causent le plus de dégâts.</p>

<p>AigisCode rend cette distinction explicite dans son analyse. Le rapport JSON sépare <code>strong_circular_dependencies</code> de <code>circular_dependencies</code>, permettant aux équipes de prioriser les cycles architecturaux qui nécessitent véritablement un refactoring tout en notant les cycles d'exécution pour information.</p>

<h2 id="why-they-are-dangerous">Pourquoi les dépendances circulaires sont dangereuses</h2>

<h3>Échecs de compilation et d'ordre de chargement</h3>

<p>Dans les langages avec une résolution de modules stricte, les dépendances circulaires peuvent provoquer des échecs purs et simples. Le système d'import de Python exécutera partiellement un module lorsqu'il rencontre un cycle, entraînant une <code>ImportError</code> ou une <code>AttributeError</code> à l'exécution lorsqu'un nom n'a pas encore été défini. En TypeScript avec la sémantique stricte des modules ES, les imports circulaires peuvent aboutir à des valeurs <code>undefined</code> au point d'utilisation car le module n'a pas fini de s'initialiser. En PHP avec l'autoloading, les dépendances circulaires peuvent provoquer des bugs subtils où une classe semble disponible mais ses dépendances n'ont pas encore été chargées.</p>

<p>Ces échecs sont notoirement difficiles à déboguer car ils dépendent de l'<em>ordre d'import</em>, qui varie selon le point d'entrée qui a déclenché le chemin de code. La même suite de tests peut réussir ou échouer selon le test qui s'exécute en premier.</p>

<h3>Cauchemars de tests</h3>

<p>Les dépendances circulaires rendent les tests unitaires extraordinairement difficiles. Si le module A dépend du module B et que le module B dépend du module A, vous ne pouvez tester aucun des deux isolément sans mocker l'autre. Cela crée une situation où la configuration de vos tests est plus complexe que le code testé, et vos mocks peuvent ne pas refléter fidèlement le comportement réel de la dépendance.</p>

<p>Prenons un exemple réel d'une application Django. Le module <code>orders</code> importe depuis <code>inventory</code> pour vérifier les niveaux de stock. Le module <code>inventory</code> importe depuis <code>orders</code> pour calculer les quantités réservées. Maintenant, pour tester unitairement le module <code>orders</code>, vous devez mocker la vérification de l'inventaire. Mais le mock doit comprendre les quantités réservées, ce qui nécessite de comprendre les commandes. Vous vous retrouvez avec des dépendances de mocks circulaires qui reflètent les dépendances circulaires du code, et vos tests deviennent fragiles, lents et peu fiables.</p>

<h3>Couplage de déploiement</h3>

<p>Dans une architecture de microservices ou de monolithe modulaire, les dépendances circulaires empêchent le déploiement indépendant. Si le service A dépend du service B et que le service B dépend du service A, vous ne pouvez déployer aucun des deux indépendamment. Chaque modification de l'un des services nécessite un déploiement coordonné des deux, éliminant l'un des principaux avantages de l'architecture modulaire.</p>

<p>Ce couplage s'étend aux frontières des équipes. Si l'équipe Alpha possède le module A et l'équipe Beta possède le module B, une dépendance circulaire signifie qu'aucune équipe ne peut livrer sans se coordonner avec l'autre. La vélocité chute. La planification de sprint devient une négociation. Et la pression pour « ajouter juste un import de plus » augmente parce que les modules sont déjà couplés de toute façon.</p>

<h3>Paralysie du refactoring</h3>

<p>Le coût le plus insidieux est peut-être que les dépendances circulaires donnent l'impression que le refactoring est impossible. Lorsque les modules sont étroitement couplés dans un cycle, modifier l'interface d'un module nécessite de modifier simultanément tous les autres dans le cycle. Il n'y a aucun moyen de le faire de manière incrémentale. Cela conduit à un état d'esprit de refactoring « big bang », où les équipes repoussent les améliorations structurelles parce que l'ampleur semble écrasante, et le cycle s'aggrave à chaque sprint.</p>

<h2 id="real-world-examples">Exemples concrets</h2>

<h3>Le cycle Django Settings</h3>

<p>Un modèle courant dans les projets Django est un cycle entre <code>settings</code>, <code>models</code> et <code>utils</code>. Settings importe depuis utils pour la résolution des chemins. Utils importe depuis models pour les requêtes de base de données. Models importe depuis settings pour les valeurs de configuration. Ce cycle à trois signifie que modifier la structure des paramètres nécessite de toucher la couche utilitaire et potentiellement la couche modèle, et vice versa.</p>

<h3>La boucle Controller-Service Node.js</h3>

<p>Dans les applications Express.js, il est courant de voir des contrôleurs importer des services et des services importer des contrôleurs (souvent pour la gestion des erreurs ou le formatage des réponses). La correction est simple : introduire un module partagé d'erreur/réponse dont les deux couches dépendent, brisant ainsi le cycle. Mais sans outil pour détecter le cycle, les équipes ne réalisent souvent pas qu'il existe jusqu'à ce qu'elles essaient d'extraire la couche de service dans une bibliothèque partagée et découvrent qu'elle ne peut pas fonctionner seule.</p>

<h3>L'enchevêtrement Event-Listener Laravel</h3>

<p>Les applications Laravel développent fréquemment des cycles entre les classes d'événements et leurs écouteurs. Un événement dans le namespace <code>Orders</code> déclenche un écouteur dans <code>Inventory</code>, qui dispatche un événement de retour vers <code>Orders</code>. Individuellement, chaque classe est propre. Ensemble, elles forment un cycle d'exécution qui peut provoquer des boucles infinies dans des conditions spécifiques et rend le flux d'événements impossible à raisonner sans visualisation de graphe.</p>

<h2 id="how-to-detect-and-fix">Comment détecter et corriger les dépendances circulaires</h2>

<h3>Détection avec AigisCode</h3>

<p>La première étape est la visibilité. Exécutez <code>aigiscode analyze /path/to/project</code> et examinez le champ <code>graph_analysis.strong_circular_dependencies</code> dans le rapport JSON. Chaque entrée liste les modules impliqués dans le cycle et les chemins d'import qui le créent. Cela vous donne une carte précise de l'emplacement des cycles et des imports qui doivent être restructurés.</p>

<h3>La correction par inversion de dépendance</h3>

<p>La correction la plus courante pour les dépendances circulaires est l'<strong>inversion de dépendance</strong>. Au lieu que le module A importe directement du module B et vice versa, vous introduisez une interface ou une classe de base abstraite dont les deux modules dépendent. Le module A dépend de l'interface. Le module B implémente l'interface. La flèche de dépendance ne pointe désormais que dans une seule direction.</p>

<p>En Python, cela signifie souvent créer un module <code>protocols.py</code> ou <code>interfaces.py</code> qui définit les contrats entre sous-systèmes. En TypeScript, cela signifie extraire les types partagés dans un répertoire <code>types/</code> depuis lequel les deux modules importent sans importer l'un de l'autre.</p>

<h3>Le pattern Médiateur</h3>

<p>Pour les cycles événementiels, le pattern médiateur est efficace. Au lieu de communiquer directement, les modules communiquent via un bus d'événements ou un médiateur partagé. Le module A dispatche un événement. Le module B l'écoute. Aucun n'importe de l'autre. Le médiateur est la seule dépendance partagée, et il ne contient aucune logique métier, juste du routage.</p>

<h3>Le pattern Extraire-et-Partager</h3>

<p>Parfois le cycle existe parce que deux modules partagent un concept qui n'a pas reçu son propre emplacement. La correction consiste à extraire le concept partagé dans un nouveau module dont les deux modules originaux dépendent. Par exemple, si <code>orders.py</code> et <code>inventory.py</code> ont tous deux besoin d'un <code>ReservationCalculator</code>, extrayez-le dans <code>reservations.py</code> et laissez les deux modules importer depuis celui-ci.</p>

<h2 id="prevention-is-cheaper-than-cure">Prévenir coûte moins cher que guérir</h2>

<p>La meilleure approche est de détecter les dépendances circulaires tôt et d'empêcher l'apparition de nouvelles. Intégrez AigisCode dans votre pipeline CI. Exécutez-le sur chaque pull request. Si une nouvelle dépendance circulaire forte apparaît, la PR devrait être signalée pour revue architecturale avant la fusion.</p>

<p>Le coût de correction d'une dépendance circulaire croît exponentiellement avec l'âge du cycle. Un cycle détecté dans une PR prend quelques minutes à corriger. Un cycle qui a grandi pendant deux ans peut nécessiter des semaines de refactoring coordonné. Les outils pour les détecter tôt existent aujourd'hui. La seule question est de savoir si votre équipe les utilise.</p>
`,
      es: `
<p>Cada base de código tiene una forma. Puede que no la vea en su editor, pero está ahí: una red de dependencias que conecta módulos, clases y funciones. Cuando esa red contiene bucles, donde el módulo A depende del módulo B y el módulo B depende a su vez del módulo A, tiene una dependencia circular. Y aunque pueda parecer una peculiaridad estructural menor, las dependencias circulares son una de las formas más costosas de deuda técnica que una base de código puede acumular.</p>

<h2 id="what-circular-dependencies-are">Qué son realmente las dependencias circulares</h2>

<p>Una dependencia circular existe cuando dos o más módulos forman un ciclo de dependencias. El caso más simple es un ciclo directo: <code>auth.py</code> importa de <code>users.py</code>, y <code>users.py</code> importa de <code>auth.py</code>. Pero los ciclos del mundo real suelen ser más largos y difíciles de detectar. Un ciclo puede involucrar cuatro o cinco módulos, cada uno luciendo limpio individualmente, pero juntos formando un bucle que los ata en una unidad inseparable.</p>

<p>Es importante distinguir entre dos tipos de ciclos. Las <strong>dependencias circulares fuertes</strong> son ciclos arquitectónicos. Existen a nivel de módulo o paquete e indican que dos subsistemas están fundamentalmente entrelazados. Las <strong>dependencias circulares totales</strong> incluyen ciclos de ejecución y orden de carga que pueden existir debido a importaciones perezosas, requires condicionales o magia del framework. Ambas vale la pena conocerlas, pero los ciclos fuertes son los que causan más daño.</p>

<p>AigisCode hace esta distinción explícita en su análisis. El informe JSON separa <code>strong_circular_dependencies</code> de <code>circular_dependencies</code>, permitiendo a los equipos priorizar los ciclos arquitectónicos que genuinamente necesitan refactorización mientras notan los ciclos de ejecución para su conocimiento.</p>

<h2 id="why-they-are-dangerous">Por qué las dependencias circulares son peligrosas</h2>

<h3>Fallos de compilación y orden de carga</h3>

<p>En lenguajes con resolución estricta de módulos, las dependencias circulares pueden causar fallos directos. El sistema de importación de Python ejecutará parcialmente un módulo cuando encuentra un ciclo, provocando <code>ImportError</code> o <code>AttributeError</code> en tiempo de ejecución cuando un nombre aún no ha sido definido. En TypeScript con semántica estricta de módulos ES, las importaciones circulares pueden resultar en valores <code>undefined</code> en el punto de uso porque el módulo no ha terminado de inicializarse. En PHP con autoloading, las dependencias circulares pueden causar bugs sutiles donde una clase parece estar disponible pero sus dependencias aún no se han cargado.</p>

<p>Estos fallos son notoriamente difíciles de depurar porque dependen del <em>orden de importación</em>, que varía según qué punto de entrada desencadenó la ruta del código. La misma suite de tests puede pasar o fallar dependiendo de qué test se ejecuta primero.</p>

<h3>Pesadillas de testing</h3>

<p>Las dependencias circulares hacen que las pruebas unitarias sean extraordinariamente difíciles. Si el módulo A depende del módulo B y el módulo B depende del módulo A, no puede probar ninguno de los dos de forma aislada sin mockear el otro. Esto crea una situación donde la configuración de sus tests es más compleja que el código que se está probando, y sus mocks pueden no reflejar con precisión el comportamiento real de la dependencia.</p>

<p>Considere un ejemplo real de una aplicación Django. El módulo <code>orders</code> importa de <code>inventory</code> para verificar niveles de stock. El módulo <code>inventory</code> importa de <code>orders</code> para calcular cantidades reservadas. Ahora, para hacer pruebas unitarias del módulo <code>orders</code>, necesita mockear la verificación de inventario. Pero el mock necesita entender las cantidades reservadas, lo que requiere entender las órdenes. Termina con dependencias de mocks circulares que reflejan las dependencias circulares del código, y sus tests se vuelven frágiles, lentos y poco confiables.</p>

<h3>Acoplamiento de despliegue</h3>

<p>En una arquitectura de microservicios o monolito modular, las dependencias circulares impiden el despliegue independiente. Si el servicio A depende del servicio B y el servicio B depende del servicio A, no puede desplegar ninguno de forma independiente. Cada cambio en cualquiera de los servicios requiere un despliegue coordinado de ambos, eliminando uno de los principales beneficios de la arquitectura modular.</p>

<p>Este acoplamiento se extiende a las fronteras de los equipos. Si el equipo Alpha es dueño del módulo A y el equipo Beta es dueño del módulo B, una dependencia circular significa que ningún equipo puede entregar sin coordinarse con el otro. La velocidad cae. La planificación de sprint se convierte en una negociación. Y la presión para "simplemente agregar una importación más" crece porque los módulos ya están acoplados de todas formas.</p>

<h3>Parálisis de refactorización</h3>

<p>Quizás el costo más insidioso es que las dependencias circulares hacen que la refactorización se sienta imposible. Cuando los módulos están estrechamente acoplados en un ciclo, cambiar la interfaz de un módulo requiere cambiar todos los demás en el ciclo simultáneamente. No hay forma de hacerlo incrementalmente. Esto lleva a una mentalidad de refactorización "big bang", donde los equipos posponen las mejoras estructurales porque el alcance se siente abrumador, y el ciclo empeora con cada sprint.</p>

<h2 id="real-world-examples">Ejemplos del mundo real</h2>

<h3>El ciclo Django Settings</h3>

<p>Un patrón común en proyectos Django es un ciclo entre <code>settings</code>, <code>models</code> y <code>utils</code>. Settings importa de utils para la resolución de rutas. Utils importa de models para consultas de base de datos. Models importa de settings para valores de configuración. Este ciclo de tres vías significa que cambiar la estructura de settings requiere tocar la capa de utilidades y potencialmente la capa de modelos, y viceversa.</p>

<h3>El bucle Controller-Service de Node.js</h3>

<p>En aplicaciones Express.js, es común ver controladores importando servicios y servicios importando controladores (a menudo para manejo de errores o formateo de respuestas). La solución es directa: introducir un módulo compartido de error/respuesta del que ambas capas dependan, rompiendo el ciclo. Pero sin una herramienta para detectar el ciclo, los equipos a menudo no se dan cuenta de que existe hasta que intentan extraer la capa de servicio en una biblioteca compartida y descubren que no puede funcionar sola.</p>

<h3>El enredo Event-Listener de Laravel</h3>

<p>Las aplicaciones Laravel frecuentemente desarrollan ciclos entre clases de eventos y sus listeners. Un evento en el namespace <code>Orders</code> desencadena un listener en <code>Inventory</code>, que despacha un evento de vuelta a <code>Orders</code>. Individualmente, cada clase está limpia. Juntas, forman un ciclo de ejecución que puede causar bucles infinitos bajo condiciones específicas y hace que el flujo de eventos sea imposible de razonar sin una visualización de grafo.</p>

<h2 id="how-to-detect-and-fix">Cómo detectar y corregir dependencias circulares</h2>

<h3>Detección con AigisCode</h3>

<p>El primer paso es la visibilidad. Ejecute <code>aigiscode analyze /path/to/project</code> y examine el campo <code>graph_analysis.strong_circular_dependencies</code> en el informe JSON. Cada entrada lista los módulos involucrados en el ciclo y las rutas de importación que lo crean. Esto le da un mapa preciso de dónde están los ciclos y qué importaciones necesitan ser reestructuradas.</p>

<h3>La corrección por inversión de dependencias</h3>

<p>La corrección más común para las dependencias circulares es la <strong>inversión de dependencias</strong>. En lugar de que el módulo A importe directamente del módulo B y viceversa, se introduce una interfaz o clase base abstracta de la que ambos módulos dependan. El módulo A depende de la interfaz. El módulo B implementa la interfaz. La flecha de dependencia ahora apunta en una sola dirección.</p>

<p>En Python, esto a menudo significa crear un módulo <code>protocols.py</code> o <code>interfaces.py</code> que define los contratos entre subsistemas. En TypeScript, significa extraer tipos compartidos en un directorio <code>types/</code> del que ambos módulos importan sin importar uno del otro.</p>

<h3>El patrón Mediador</h3>

<p>Para ciclos basados en eventos, el patrón mediador es efectivo. En lugar de comunicarse directamente, los módulos se comunican a través de un bus de eventos o mediador compartido. El módulo A despacha un evento. El módulo B lo escucha. Ninguno importa del otro. El mediador es la única dependencia compartida, y no contiene lógica de negocio, solo enrutamiento.</p>

<h3>El patrón Extraer-y-Compartir</h3>

<p>A veces el ciclo existe porque dos módulos comparten un concepto que no tiene su propio hogar. La corrección es extraer el concepto compartido en un nuevo módulo del que ambos módulos originales dependan. Por ejemplo, si <code>orders.py</code> e <code>inventory.py</code> necesitan un <code>ReservationCalculator</code>, extráigalo a <code>reservations.py</code> y deje que ambos módulos importen de él.</p>

<h2 id="prevention-is-cheaper-than-cure">Prevenir es más barato que curar</h2>

<p>El mejor enfoque es detectar las dependencias circulares temprano y prevenir que se formen nuevas. Integre AigisCode en su pipeline de CI. Ejecútelo en cada pull request. Si aparece una nueva dependencia circular fuerte, la PR debe ser marcada para revisión arquitectónica antes de la fusión.</p>

<p>El costo de corregir una dependencia circular crece exponencialmente con la edad del ciclo. Un ciclo detectado en una PR toma minutos para corregir. Un ciclo que ha estado creciendo durante dos años puede requerir semanas de refactorización coordinada. Las herramientas para detectarlos temprano existen hoy. La única pregunta es si su equipo las utiliza.</p>
`,
      zh: `
<p>每个代码库都有自己的形态。你可能在编辑器中看不到它，但它确实存在：一个连接模块、类和函数的依赖网络。当这个网络包含循环，即模块 A 依赖模块 B，而模块 B 又依赖回模块 A 时，你就有了循环依赖。虽然它看起来可能只是一个小小的结构怪癖，但循环依赖是代码库可能积累的最昂贵的技术债务形式之一。</p>

<h2 id="what-circular-dependencies-are">循环依赖到底是什么</h2>

<p>当两个或更多模块形成依赖循环时，就存在循环依赖。最简单的情况是直接循环：<code>auth.py</code> 从 <code>users.py</code> 导入，而 <code>users.py</code> 从 <code>auth.py</code> 导入。但现实世界中的循环往往更长、更难发现。一个循环可能涉及四五个模块，每个单独看起来都很干净，但组合在一起却形成了一个将它们绑定为不可分割单元的环路。</p>

<p>区分两种类型的循环很重要。<strong>强循环依赖</strong>是架构层面的循环。它们存在于模块或包级别，表明两个子系统从根本上纠缠在一起。<strong>总循环依赖</strong>包括运行时和加载顺序循环，这些可能由于延迟导入、条件性 require 或框架魔法而存在。两者都值得了解，但造成最大伤害的是强循环。</p>

<p>AigisCode 在其分析中明确区分了这一点。JSON 报告将 <code>strong_circular_dependencies</code> 与 <code>circular_dependencies</code> 分开，允许团队优先处理真正需要重构的架构循环，同时记录运行时循环以供参考。</p>

<h2 id="why-they-are-dangerous">为什么循环依赖是危险的</h2>

<h3>编译和加载顺序失败</h3>

<p>在具有严格模块解析的语言中，循环依赖可能导致直接失败。Python 的导入系统在遇到循环时会部分执行模块，当名称尚未定义时导致运行时的 <code>ImportError</code> 或 <code>AttributeError</code>。在使用严格 ES 模块语义的 TypeScript 中，循环导入可能导致使用点出现 <code>undefined</code> 值，因为模块尚未完成初始化。在使用自动加载的 PHP 中，循环依赖可能导致微妙的 bug，其中一个类看起来可用，但其依赖项尚未加载。</p>

<p>这些失败出了名的难以调试，因为它们取决于<em>导入顺序</em>，而导入顺序因触发代码路径的入口点不同而不同。同一个测试套件可能根据哪个测试先运行而通过或失败。</p>

<h3>测试噩梦</h3>

<p>循环依赖使单元测试变得异常困难。如果模块 A 依赖模块 B 而模块 B 依赖模块 A，你无法在不 mock 另一个的情况下独立测试任何一个。这创造了一种情况：你的测试配置比被测试的代码更复杂，而你的 mock 可能无法准确反映依赖项的真实行为。</p>

<p>考虑一个来自 Django 应用的真实例子。<code>orders</code> 模块从 <code>inventory</code> 导入以检查库存水平。<code>inventory</code> 模块从 <code>orders</code> 导入以计算预留数量。现在，要对 <code>orders</code> 模块进行单元测试，你需要 mock 库存检查。但 mock 需要理解预留数量，这又需要理解订单。你最终得到的是镜像代码循环依赖的循环 mock 依赖，你的测试变得脆弱、缓慢且不可靠。</p>

<h3>部署耦合</h3>

<p>在微服务或模块化单体架构中，循环依赖阻止独立部署。如果服务 A 依赖服务 B 而服务 B 依赖服务 A，你无法独立部署任何一个。对任一服务的每次更改都需要协调部署两者，消除了模块化架构的主要优势之一。</p>

<p>这种耦合延伸到团队边界。如果 Alpha 团队拥有模块 A，Beta 团队拥有模块 B，循环依赖意味着两个团队都无法在不与对方协调的情况下发布。速度下降。Sprint 计划变成了谈判。而"再加一个导入"的压力增大，因为模块本来就已经耦合了。</p>

<h3>重构瘫痪</h3>

<p>也许最阴险的代价是循环依赖让重构感觉不可能。当模块在循环中紧密耦合时，更改一个模块的接口需要同时更改循环中的所有其他模块。没有办法增量完成。这导致了"大爆炸"重构心态，团队因为范围感觉太大而推迟结构改进，循环随着每个 sprint 变得更糟。</p>

<h2 id="real-world-examples">真实世界的例子</h2>

<h3>Django Settings 循环</h3>

<p>Django 项目中一个常见的模式是 <code>settings</code>、<code>models</code> 和 <code>utils</code> 之间的循环。Settings 从 utils 导入用于路径解析。Utils 从 models 导入用于数据库查询。Models 从 settings 导入用于配置值。这个三方循环意味着更改 settings 结构需要触及工具层，可能还有模型层，反之亦然。</p>

<h3>Node.js Controller-Service 循环</h3>

<p>在 Express.js 应用中，常见的是控制器导入服务，服务导入控制器（通常用于错误处理或响应格式化）。修复很简单：引入一个两层都依赖的共享错误/响应模块，打破循环。但如果没有检测循环的工具，团队往往在尝试将服务层提取到共享库时才意识到它的存在，并发现它无法独立运行。</p>

<h3>Laravel Event-Listener 纠缠</h3>

<p>Laravel 应用经常在事件类和其监听器之间发展出循环。<code>Orders</code> 命名空间中的事件触发 <code>Inventory</code> 中的监听器，后者又向 <code>Orders</code> 发送事件。单独来看，每个类都很干净。但组合在一起，它们形成了一个运行时循环，在特定条件下可能导致无限循环，并使得没有图形可视化就无法推理事件流。</p>

<h2 id="how-to-detect-and-fix">如何检测和修复循环依赖</h2>

<h3>使用 AigisCode 检测</h3>

<p>第一步是可见性。运行 <code>aigiscode analyze /path/to/project</code> 并检查 JSON 报告中的 <code>graph_analysis.strong_circular_dependencies</code> 字段。每个条目列出循环中涉及的模块和创建它的导入路径。这为你提供了循环位置和需要重构的导入的精确地图。</p>

<h3>依赖反转修复</h3>

<p>循环依赖最常见的修复方法是<strong>依赖反转</strong>。不是模块 A 直接从模块 B 导入，反之亦然，而是引入一个两个模块都依赖的接口或抽象基类。模块 A 依赖接口。模块 B 实现接口。依赖箭头现在只指向一个方向。</p>

<p>在 Python 中，这通常意味着创建一个 <code>protocols.py</code> 或 <code>interfaces.py</code> 模块来定义子系统之间的契约。在 TypeScript 中，这意味着将共享类型提取到 <code>types/</code> 目录中，两个模块从中导入而不相互导入。</p>

<h3>中介者模式</h3>

<p>对于事件驱动的循环，中介者模式很有效。模块不直接通信，而是通过共享的事件总线或中介者通信。模块 A 发送事件。模块 B 监听它。两者都不从对方导入。中介者是唯一的共享依赖，它不包含业务逻辑，只有路由。</p>

<h3>提取并共享模式</h3>

<p>有时循环存在是因为两个模块共享了一个没有自己归属的概念。修复方法是将共享概念提取到一个新模块中，两个原始模块都依赖它。例如，如果 <code>orders.py</code> 和 <code>inventory.py</code> 都需要 <code>ReservationCalculator</code>，将它提取到 <code>reservations.py</code> 中，让两个模块都从中导入。</p>

<h2 id="prevention-is-cheaper-than-cure">预防比治疗更便宜</h2>

<p>最好的方法是及早检测循环依赖并防止新的形成。将 AigisCode 集成到你的 CI 流水线中。在每个 pull request 上运行它。如果出现新的强循环依赖，PR 应在合并前被标记进行架构审查。</p>

<p>修复循环依赖的成本随循环的年龄呈指数增长。在 PR 中发现的循环只需几分钟修复。一个增长了两年的循环可能需要数周的协调重构。及早发现它们的工具今天就存在。唯一的问题是你的团队是否使用它们。</p>
`,
      hi: `

<p>हर कोडबेस का एक आकार होता है। आप इसे अपने एडिटर में नहीं देख सकते, लेकिन यह वहां है: मॉड्यूल, क्लासेज और फंक्शंस को जोड़ने वाली निर्भरताओं का एक जाल। जब उस जाल में लूप होते हैं, जहां मॉड्यूल A मॉड्यूल B पर निर्भर करता है और मॉड्यूल B वापस मॉड्यूल A पर निर्भर करता है, तो आपके पास एक चक्रीय निर्भरता है। और हालांकि यह एक मामूली संरचनात्मक विशेषता लग सकती है, चक्रीय निर्भरताएं तकनीकी ऋण के सबसे महंगे रूपों में से एक हैं जो एक कोडबेस जमा कर सकता है।</p>

<h2 id="what-circular-dependencies-are">चक्रीय निर्भरताएं वास्तव में क्या हैं</h2>

<p>चक्रीय निर्भरता तब मौजूद होती है जब दो या अधिक मॉड्यूल एक निर्भरता चक्र बनाते हैं। सबसे सरल मामला एक प्रत्यक्ष चक्र है: <code>auth.py</code> <code>users.py</code> से आयात करता है, और <code>users.py</code> <code>auth.py</code> से आयात करता है। लेकिन वास्तविक दुनिया के चक्र अक्सर लंबे और पहचानने में कठिन होते हैं। एक चक्र में चार या पांच मॉड्यूल शामिल हो सकते हैं, जिनमें से प्रत्येक व्यक्तिगत रूप से साफ दिखता है, लेकिन एक साथ वे एक लूप बनाते हैं जो उन्हें एक अविभाज्य इकाई में बांधता है।</p>

<p>दो प्रकार के चक्रों के बीच अंतर करना महत्वपूर्ण है। <strong>मजबूत चक्रीय निर्भरताएं</strong> आर्किटेक्चरल चक्र हैं। वे मॉड्यूल या पैकेज स्तर पर मौजूद हैं और इंगित करती हैं कि दो उपतंत्र मूलभूत रूप से उलझे हुए हैं। <strong>कुल चक्रीय निर्भरताएं</strong> में रनटाइम और लोड-ऑर्डर चक्र शामिल हैं जो lazy imports, सशर्त requires, या फ्रेमवर्क मैजिक के कारण मौजूद हो सकते हैं। दोनों के बारे में जानना उचित है, लेकिन मजबूत चक्र वे हैं जो सबसे अधिक नुकसान पहुंचाते हैं।</p>

<p>AigisCode अपने विश्लेषण में यह भेद स्पष्ट रूप से करता है। JSON रिपोर्ट <code>strong_circular_dependencies</code> को <code>circular_dependencies</code> से अलग करती है, जिससे टीमें उन आर्किटेक्चरल चक्रों को प्राथमिकता दे सकती हैं जिन्हें वास्तव में रीफैक्टरिंग की आवश्यकता है, जबकि रनटाइम चक्रों को जागरूकता के लिए नोट करती हैं।</p>

<h2 id="why-they-are-dangerous">चक्रीय निर्भरताएं खतरनाक क्यों हैं</h2>

<h3>संकलन और लोड-ऑर्डर विफलताएं</h3>

<p>सख्त मॉड्यूल रिज़ॉल्यूशन वाली भाषाओं में, चक्रीय निर्भरताएं सीधी विफलताओं का कारण बन सकती हैं। Python का आयात सिस्टम एक मॉड्यूल को आंशिक रूप से निष्पादित करेगा जब उसे चक्र का सामना होता है, जिससे रनटाइम पर <code>ImportError</code> या <code>AttributeError</code> होता है जब कोई नाम अभी तक परिभाषित नहीं हुआ है। सख्त ES मॉड्यूल सिमेंटिक्स वाले TypeScript में, चक्रीय आयात उपयोग के बिंदु पर <code>undefined</code> मान उत्पन्न कर सकते हैं क्योंकि मॉड्यूल ने अभी तक आरंभीकरण पूरा नहीं किया है। ऑटोलोडिंग वाले PHP में, चक्रीय निर्भरताएं सूक्ष्म बग पैदा कर सकती हैं जहां एक क्लास उपलब्ध दिखाई देती है लेकिन उसकी निर्भरताएं अभी तक लोड नहीं हुई हैं।</p>

<p>ये विफलताएं डीबग करने में कुख्यात रूप से कठिन हैं क्योंकि वे <em>आयात क्रम</em> पर निर्भर करती हैं, जो इस आधार पर भिन्न होता है कि किस एंट्री पॉइंट ने कोड पथ को ट्रिगर किया। एक ही टेस्ट सूट इस आधार पर पास या फेल हो सकता है कि कौन सा टेस्ट पहले चलता है।</p>

<h3>परीक्षण के दुःस्वप्न</h3>

<p>चक्रीय निर्भरताएं यूनिट परीक्षण को असाधारण रूप से कठिन बना देती हैं। यदि मॉड्यूल A मॉड्यूल B पर निर्भर है और मॉड्यूल B मॉड्यूल A पर निर्भर है, तो आप दूसरे को मॉक किए बिना किसी को भी अलग से परीक्षण नहीं कर सकते। यह एक ऐसी स्थिति बनाता है जहां आपका टेस्ट सेटअप परीक्षण किए जा रहे कोड से अधिक जटिल है, और आपके मॉक निर्भरता के वास्तविक व्यवहार को सटीक रूप से प्रतिबिंबित नहीं कर सकते।</p>

<p>एक Django एप्लिकेशन से एक वास्तविक उदाहरण पर विचार करें। <code>orders</code> मॉड्यूल स्टॉक स्तरों की जांच के लिए <code>inventory</code> से आयात करता है। <code>inventory</code> मॉड्यूल आरक्षित मात्राओं की गणना के लिए <code>orders</code> से आयात करता है। अब, <code>orders</code> मॉड्यूल का यूनिट टेस्ट करने के लिए, आपको इन्वेंट्री चेक को मॉक करना होगा। लेकिन मॉक को आरक्षित मात्राओं को समझने की आवश्यकता है, जिसके लिए ऑर्डर की समझ आवश्यक है। आप चक्रीय मॉक निर्भरताओं के साथ समाप्त होते हैं जो चक्रीय कोड निर्भरताओं को प्रतिबिंबित करती हैं, और आपके परीक्षण नाजुक, धीमे और अविश्वसनीय हो जाते हैं।</p>

<h3>तैनाती युग्मन</h3>

<p>माइक्रोसर्विस या मॉड्यूलर मोनोलिथ आर्किटेक्चर में, चक्रीय निर्भरताएं स्वतंत्र तैनाती को रोकती हैं। यदि सेवा A सेवा B पर निर्भर है और सेवा B सेवा A पर निर्भर है, तो आप किसी को भी स्वतंत्र रूप से तैनात नहीं कर सकते। किसी भी सेवा में हर बदलाव के लिए दोनों की समन्वित तैनाती की आवश्यकता होती है, जो मॉड्यूलर आर्किटेक्चर के प्राथमिक लाभों में से एक को समाप्त कर देती है।</p>

<p>यह युग्मन टीम सीमाओं तक फैलता है। यदि टीम अल्फा मॉड्यूल A की मालिक है और टीम बीटा मॉड्यूल B की मालिक है, तो चक्रीय निर्भरता का मतलब है कि कोई भी टीम दूसरे के साथ समन्वय किए बिना शिप नहीं कर सकती। वेग गिरता है। स्प्रिंट प्लानिंग एक बातचीत बन जाती है। और "बस एक और import जोड़ दो" का दबाव बढ़ता है क्योंकि मॉड्यूल पहले से ही युग्मित हैं।</p>

<h3>रीफैक्टरिंग पक्षाघात</h3>

<p>शायद सबसे कपटी लागत यह है कि चक्रीय निर्भरताएं रीफैक्टरिंग को असंभव महसूस कराती हैं। जब मॉड्यूल एक चक्र में कसकर युग्मित होते हैं, तो एक मॉड्यूल के इंटरफेस को बदलने के लिए चक्र में अन्य सभी को एक साथ बदलना आवश्यक होता है। इसे क्रमिक रूप से करने का कोई तरीका नहीं है। यह "बिग बैंग" रीफैक्टरिंग मानसिकता की ओर ले जाता है, जहां टीमें संरचनात्मक सुधारों को स्थगित करती हैं क्योंकि दायरा भारी लगता है, और चक्र हर स्प्रिंट के साथ बदतर होता जाता है।</p>

<h2 id="real-world-examples">वास्तविक दुनिया के उदाहरण</h2>

<h3>Django सेटिंग्स चक्र</h3>

<p>Django प्रोजेक्ट्स में एक सामान्य पैटर्न <code>settings</code>, <code>models</code> और <code>utils</code> के बीच एक चक्र है। Settings पथ रिज़ॉल्यूशन के लिए utils से आयात करता है। Utils डेटाबेस क्वेरी के लिए models से आयात करता है। Models कॉन्फ़िगरेशन मानों के लिए settings से आयात करता है। यह तीन-तरफा चक्र का मतलब है कि सेटिंग्स संरचना को बदलने के लिए यूटिलिटी लेयर और संभावित रूप से मॉडल लेयर को छूना आवश्यक है, और इसके विपरीत।</p>

<h3>Node.js Controller-Service लूप</h3>

<p>Express.js एप्लिकेशन में, कंट्रोलर्स द्वारा सर्विसेज को आयात करना और सर्विसेज द्वारा कंट्रोलर्स को आयात करना (अक्सर त्रुटि प्रबंधन या प्रतिक्रिया फॉर्मेटिंग के लिए) आम है। समाधान सीधा है: एक साझा त्रुटि/प्रतिक्रिया मॉड्यूल पेश करें जिस पर दोनों परतें निर्भर हों, चक्र को तोड़ दें। लेकिन चक्र का पता लगाने के लिए किसी उपकरण के बिना, टीमों को अक्सर यह एहसास नहीं होता कि यह मौजूद है जब तक वे सेवा परत को साझा लाइब्रेरी में निकालने का प्रयास नहीं करते और पाते हैं कि यह अकेले खड़ा नहीं हो सकता।</p>

<h3>Laravel Event-Listener उलझन</h3>

<p>Laravel एप्लिकेशन अक्सर ईवेंट क्लासेज और उनके लिसनर्स के बीच चक्र विकसित करते हैं। <code>Orders</code> नेमस्पेस में एक ईवेंट <code>Inventory</code> में एक लिसनर को ट्रिगर करता है, जो <code>Orders</code> को वापस एक ईवेंट भेजता है। व्यक्तिगत रूप से, प्रत्येक क्लास साफ है। एक साथ, वे एक रनटाइम चक्र बनाती हैं जो विशिष्ट स्थितियों में अनंत लूप पैदा कर सकता है और ग्राफ विज़ुअलाइज़ेशन के बिना ईवेंट प्रवाह के बारे में तर्क करना असंभव बना देता है।</p>

<h2 id="how-to-detect-and-fix">चक्रीय निर्भरताओं का पता कैसे लगाएं और कैसे ठीक करें</h2>

<h3>AigisCode के साथ पहचान</h3>

<p>पहला कदम दृश्यता है। <code>aigiscode analyze /path/to/project</code> चलाएं और JSON रिपोर्ट में <code>graph_analysis.strong_circular_dependencies</code> फील्ड की जांच करें। प्रत्येक प्रविष्टि चक्र में शामिल मॉड्यूल और इसे बनाने वाले आयात पथों को सूचीबद्ध करती है। यह आपको एक सटीक नक्शा देता है कि चक्र कहां हैं और किन आयातों को पुनर्गठित करने की आवश्यकता है।</p>

<h3>डिपेंडेंसी इनवर्शन समाधान</h3>

<p>चक्रीय निर्भरताओं के लिए सबसे आम समाधान <strong>डिपेंडेंसी इनवर्शन</strong> है। मॉड्यूल A द्वारा सीधे मॉड्यूल B से आयात करने और इसके विपरीत के बजाय, आप एक इंटरफेस या अमूर्त आधार वर्ग पेश करते हैं जिस पर दोनों मॉड्यूल निर्भर करते हैं। मॉड्यूल A इंटरफेस पर निर्भर करता है। मॉड्यूल B इंटरफेस को लागू करता है। निर्भरता का तीर अब केवल एक दिशा में इंगित करता है।</p>

<p>Python में, इसका अक्सर मतलब है एक <code>protocols.py</code> या <code>interfaces.py</code> मॉड्यूल बनाना जो उपतंत्रों के बीच अनुबंधों को परिभाषित करता है। TypeScript में, इसका मतलब है साझा प्रकारों को एक <code>types/</code> डायरेक्टरी में निकालना जिससे दोनों मॉड्यूल एक दूसरे से आयात किए बिना आयात करें।</p>

<h3>मीडिएटर पैटर्न</h3>

<p>ईवेंट-चालित चक्रों के लिए, मीडिएटर पैटर्न प्रभावी है। मॉड्यूलों द्वारा सीधे संवाद करने के बजाय, वे एक साझा ईवेंट बस या मीडिएटर के माध्यम से संवाद करते हैं। मॉड्यूल A एक ईवेंट भेजता है। मॉड्यूल B इसे सुनता है। कोई भी दूसरे से आयात नहीं करता। मीडिएटर एकमात्र साझा निर्भरता है, और इसमें कोई व्यावसायिक तर्क नहीं है, केवल रूटिंग।</p>

<h3>एक्सट्रैक्ट-एंड-शेयर पैटर्न</h3>

<p>कभी-कभी चक्र इसलिए मौजूद होता है क्योंकि दो मॉड्यूल एक ऐसी अवधारणा साझा करते हैं जिसे अपना घर नहीं दिया गया है। समाधान साझा अवधारणा को एक नए मॉड्यूल में निकालना है जिस पर दोनों मूल मॉड्यूल निर्भर करते हैं। उदाहरण के लिए, यदि <code>orders.py</code> और <code>inventory.py</code> दोनों को <code>ReservationCalculator</code> की आवश्यकता है, तो इसे <code>reservations.py</code> में निकालें और दोनों मॉड्यूल को इससे आयात करने दें।</p>

<h2 id="prevention-is-cheaper-than-cure">रोकथाम इलाज से सस्ती है</h2>

<p>सबसे अच्छा दृष्टिकोण चक्रीय निर्भरताओं का जल्दी पता लगाना और नए बनने से रोकना है। AigisCode को अपनी CI पाइपलाइन में एकीकृत करें। हर पुल रिक्वेस्ट पर इसे चलाएं। यदि कोई नई मजबूत चक्रीय निर्भरता दिखाई देती है, तो PR को मर्ज करने से पहले आर्किटेक्चरल समीक्षा के लिए चिह्नित किया जाना चाहिए।</p>

<p>चक्रीय निर्भरता को ठीक करने की लागत चक्र की आयु के साथ तेजी से बढ़ती है। PR में पकड़ा गया चक्र मिनटों में ठीक हो जाता है। दो साल से बढ़ रहे चक्र को हफ्तों की समन्वित रीफैक्टरिंग की आवश्यकता हो सकती है। उन्हें जल्दी पकड़ने के उपकरण आज मौजूद हैं। एकमात्र सवाल यह है कि क्या आपकी टीम उनका उपयोग करती है।</p>
`,
      pt: `

<p>Toda base de código tem uma forma. Você pode não vê-la no seu editor, mas ela está lá: uma teia de dependências conectando módulos, classes e funções. Quando essa teia contém laços, onde o módulo A depende do módulo B e o módulo B depende de volta do módulo A, você tem uma dependência circular. E embora possa parecer uma peculiaridade estrutural menor, dependências circulares são uma das formas mais caras de dívida técnica que uma base de código pode acumular.</p>

<h2 id="what-circular-dependencies-are">O que São Dependências Circulares</h2>

<p>Uma dependência circular existe quando dois ou mais módulos formam um ciclo de dependência. O caso mais simples é um ciclo direto: <code>auth.py</code> importa de <code>users.py</code>, e <code>users.py</code> importa de <code>auth.py</code>. Mas ciclos do mundo real são frequentemente mais longos e difíceis de identificar. Um ciclo pode envolver quatro ou cinco módulos, cada um parecendo limpo individualmente, mas juntos formando um laço que os une numa unidade inseparável.</p>

<p>É importante distinguir entre dois tipos de ciclos. <strong>Dependências circulares fortes</strong> são ciclos arquiteturais. Existem no nível de módulo ou pacote e indicam que dois subsistemas estão fundamentalmente entrelaçados. <strong>Dependências circulares totais</strong> incluem ciclos de runtime e ordem de carregamento que podem existir devido a lazy imports, requires condicionais ou magia de framework. Ambos vale a pena conhecer, mas os ciclos fortes são os que causam mais dano.</p>

<p>O AigisCode torna essa distinção explícita em sua análise. O relatório JSON separa <code>strong_circular_dependencies</code> de <code>circular_dependencies</code>, permitindo que as equipes priorizem os ciclos arquiteturais que genuinamente precisam de refatoração enquanto notam os ciclos de runtime para conscientização.</p>

<h2 id="why-they-are-dangerous">Por que Dependências Circulares São Perigosas</h2>

<h3>Falhas de Compilação e Ordem de Carregamento</h3>

<p>Em linguagens com resolução estrita de módulos, dependências circulares podem causar falhas diretas. O sistema de importação do Python executará parcialmente um módulo ao encontrar um ciclo, levando a <code>ImportError</code> ou <code>AttributeError</code> em tempo de execução quando um nome ainda não foi definido. Em TypeScript com semântica estrita de módulos ES, importações circulares podem resultar em valores <code>undefined</code> no ponto de uso porque o módulo ainda não terminou de inicializar. Em PHP com autoloading, dependências circulares podem causar bugs sutis onde uma classe parece estar disponível mas suas dependências ainda não foram carregadas.</p>

<p>Essas falhas são notoriamente difíceis de depurar porque dependem da <em>ordem de importação</em>, que varia com base em qual ponto de entrada acionou o caminho do código. O mesmo conjunto de testes pode passar ou falhar dependendo de qual teste é executado primeiro.</p>

<h3>Pesadelos de Teste</h3>

<p>Dependências circulares tornam o teste unitário extraordinariamente difícil. Se o módulo A depende do módulo B e o módulo B depende do módulo A, você não pode testar nenhum dos dois isoladamente sem fazer mock do outro. Isso cria uma situação onde a preparação do seu teste é mais complexa que o código sendo testado, e seus mocks podem não refletir com precisão o comportamento real da dependência.</p>

<p>Considere um exemplo real de uma aplicação Django. O módulo <code>orders</code> importa de <code>inventory</code> para verificar níveis de estoque. O módulo <code>inventory</code> importa de <code>orders</code> para calcular quantidades reservadas. Agora, para testar unitariamente o módulo <code>orders</code>, você precisa fazer mock da verificação de inventário. Mas o mock precisa entender quantidades reservadas, o que requer entender pedidos. Você acaba com dependências circulares de mock que espelham as dependências circulares do código, e seus testes se tornam frágeis, lentos e não confiáveis.</p>

<h3>Acoplamento de Implantação</h3>

<p>Numa arquitetura de microsserviços ou monolito modular, dependências circulares impedem a implantação independente. Se o serviço A depende do serviço B e o serviço B depende do serviço A, você não pode implantar nenhum dos dois independentemente. Toda alteração em qualquer serviço requer implantação coordenada de ambos, eliminando um dos benefícios primários da arquitetura modular.</p>

<p>Esse acoplamento se estende às fronteiras das equipes. Se a equipe Alfa é dona do módulo A e a equipe Beta é dona do módulo B, uma dependência circular significa que nenhuma equipe pode entregar sem coordenar com a outra. A velocidade cai. O planejamento de sprint se torna uma negociação. E a pressão para "apenas adicionar mais um import" cresce porque os módulos já estão acoplados de qualquer forma.</p>

<h3>Paralisia de Refatoração</h3>

<p>Talvez o custo mais insidioso seja que dependências circulares fazem a refatoração parecer impossível. Quando módulos estão fortemente acoplados em um ciclo, alterar a interface de um módulo requer alterar todos os outros no ciclo simultaneamente. Não há como fazer isso incrementalmente. Isso leva a uma mentalidade de refatoração "big bang", onde equipes adiam melhorias estruturais porque o escopo parece esmagador, e o ciclo piora a cada sprint.</p>

<h2 id="real-world-examples">Exemplos do Mundo Real</h2>

<h3>O Ciclo de Settings do Django</h3>

<p>Um padrão comum em projetos Django é um ciclo entre <code>settings</code>, <code>models</code> e <code>utils</code>. Settings importa de utils para resolução de caminhos. Utils importa de models para consultas ao banco de dados. Models importa de settings para valores de configuração. Este ciclo de três vias significa que alterar a estrutura de settings requer tocar na camada de utilitários e potencialmente na camada de modelos, e vice-versa.</p>

<h3>O Loop Controller-Service do Node.js</h3>

<p>Em aplicações Express.js, é comum ver controllers importando services e services importando controllers (frequentemente para tratamento de erros ou formatação de resposta). A correção é direta: introduzir um módulo compartilhado de erro/resposta do qual ambas as camadas dependam, quebrando o ciclo. Mas sem uma ferramenta para detectar o ciclo, as equipes frequentemente não percebem que ele existe até tentarem extrair a camada de serviço numa biblioteca compartilhada e descobrirem que ela não funciona sozinha.</p>

<h3>O Emaranhado Event-Listener do Laravel</h3>

<p>Aplicações Laravel frequentemente desenvolvem ciclos entre classes de eventos e seus listeners. Um evento no namespace <code>Orders</code> aciona um listener em <code>Inventory</code>, que despacha um evento de volta para <code>Orders</code>. Individualmente, cada classe é limpa. Juntas, formam um ciclo de runtime que pode causar loops infinitos sob condições específicas e torna impossível raciocinar sobre o fluxo de eventos sem uma visualização de grafo.</p>

<h2 id="how-to-detect-and-fix">Como Detectar e Corrigir Dependências Circulares</h2>

<h3>Detecção com AigisCode</h3>

<p>O primeiro passo é visibilidade. Execute <code>aigiscode analyze /path/to/project</code> e examine o campo <code>graph_analysis.strong_circular_dependencies</code> no relatório JSON. Cada entrada lista os módulos envolvidos no ciclo e os caminhos de importação que o criam. Isso fornece um mapa preciso de onde estão os ciclos e quais importações precisam ser reestruturadas.</p>

<h3>A Correção por Inversão de Dependência</h3>

<p>A correção mais comum para dependências circulares é a <strong>inversão de dependência</strong>. Em vez do módulo A importar diretamente do módulo B e vice-versa, você introduz uma interface ou classe base abstrata da qual ambos os módulos dependem. O módulo A depende da interface. O módulo B implementa a interface. A seta de dependência agora aponta em apenas uma direção.</p>

<p>Em Python, isso frequentemente significa criar um módulo <code>protocols.py</code> ou <code>interfaces.py</code> que define os contratos entre subsistemas. Em TypeScript, significa extrair tipos compartilhados num diretório <code>types/</code> do qual ambos os módulos importam sem importar um do outro.</p>

<h3>O Padrão Mediator</h3>

<p>Para ciclos orientados a eventos, o padrão mediator é eficaz. Em vez de módulos se comunicarem diretamente, eles se comunicam através de um barramento de eventos compartilhado ou mediator. O módulo A despacha um evento. O módulo B escuta por ele. Nenhum importa do outro. O mediator é a única dependência compartilhada, e não contém lógica de negócios, apenas roteamento.</p>

<h3>O Padrão Extrair-e-Compartilhar</h3>

<p>Às vezes o ciclo existe porque dois módulos compartilham um conceito que não recebeu seu próprio lar. A correção é extrair o conceito compartilhado num novo módulo do qual ambos os módulos originais dependam. Por exemplo, se <code>orders.py</code> e <code>inventory.py</code> ambos precisam de um <code>ReservationCalculator</code>, extraia-o para <code>reservations.py</code> e deixe ambos os módulos importar dele.</p>

<h2 id="prevention-is-cheaper-than-cure">Prevenção É Mais Barata que a Cura</h2>

<p>A melhor abordagem é detectar dependências circulares cedo e prevenir que novas se formem. Integre o AigisCode no seu pipeline de CI. Execute-o em cada pull request. Se uma nova dependência circular forte aparecer, o PR deve ser sinalizado para revisão arquitetural antes do merge.</p>

<p>O custo de corrigir uma dependência circular cresce exponencialmente com a idade do ciclo. Um ciclo detectado num PR leva minutos para corrigir. Um ciclo que vem crescendo há dois anos pode levar semanas de refatoração coordenada. As ferramentas para detectá-los cedo existem hoje. A única questão é se a sua equipe as utiliza.</p>
`,
      ar: `

<p>لكل قاعدة شيفرة شكل. قد لا تراه في محررك، لكنه موجود: شبكة من التبعيات تربط الوحدات والفئات والدوال. عندما تحتوي تلك الشبكة على حلقات، حيث تعتمد الوحدة A على الوحدة B وتعتمد الوحدة B بدورها على الوحدة A، يكون لديك تبعية دائرية. وبينما قد تبدو كخلل هيكلي بسيط، فإن التبعيات الدائرية هي واحدة من أكثر أشكال الديون التقنية تكلفة التي يمكن لقاعدة الشيفرة أن تراكمها.</p>

<h2 id="what-circular-dependencies-are">ما هي التبعيات الدائرية حقاً</h2>

<p>توجد تبعية دائرية عندما تشكل وحدتان أو أكثر دورة تبعية. أبسط حالة هي دورة مباشرة: <code>auth.py</code> يستورد من <code>users.py</code>، و <code>users.py</code> يستورد من <code>auth.py</code>. لكن الدورات في الواقع غالباً ما تكون أطول وأصعب في الاكتشاف. قد تتضمن الدورة أربع أو خمس وحدات، كل منها يبدو نظيفاً بمفرده، لكنها معاً تشكل حلقة تربطها في وحدة لا يمكن فصلها.</p>

<p>من المهم التمييز بين نوعين من الدورات. <strong>التبعيات الدائرية القوية</strong> هي دورات معمارية. توجد على مستوى الوحدة أو الحزمة وتشير إلى أن نظامين فرعيين متشابكان جوهرياً. <strong>التبعيات الدائرية الكلية</strong> تشمل دورات وقت التشغيل وترتيب التحميل التي قد توجد بسبب الاستيرادات الكسولة أو المتطلبات الشرطية أو سحر إطار العمل. كلاهما يستحق المعرفة، لكن الدورات القوية هي التي تسبب أكبر ضرر.</p>

<p>يجعل AigisCode هذا التمييز صريحاً في تحليله. يفصل تقرير JSON بين <code>strong_circular_dependencies</code> و <code>circular_dependencies</code>، مما يتيح للفرق إعطاء الأولوية للدورات المعمارية التي تحتاج فعلاً إلى إعادة هيكلة مع ملاحظة دورات وقت التشغيل للتوعية.</p>

<h2 id="why-they-are-dangerous">لماذا التبعيات الدائرية خطيرة</h2>

<h3>فشل التجميع وترتيب التحميل</h3>

<p>في اللغات ذات حل الوحدات الصارم، يمكن للتبعيات الدائرية أن تسبب فشلاً مباشراً. سينفذ نظام استيراد Python وحدة جزئياً عند مواجهة دورة، مما يؤدي إلى <code>ImportError</code> أو <code>AttributeError</code> في وقت التشغيل عندما لم يتم تعريف اسم بعد. في TypeScript مع دلالات وحدات ES الصارمة، يمكن أن تؤدي الاستيرادات الدائرية إلى قيم <code>undefined</code> عند نقطة الاستخدام لأن الوحدة لم تنتهِ من التهيئة. في PHP مع التحميل التلقائي، يمكن للتبعيات الدائرية أن تسبب أخطاء دقيقة حيث تبدو فئة متاحة لكن تبعياتها لم يتم تحميلها بعد.</p>

<p>هذه الإخفاقات صعبة التتبع بشكل سيئ السمعة لأنها تعتمد على <em>ترتيب الاستيراد</em>، الذي يتغير بناءً على نقطة الدخول التي أطلقت مسار الشيفرة. قد ينجح أو يفشل نفس مجموعة الاختبارات اعتماداً على أي اختبار يعمل أولاً.</p>

<h3>كوابيس الاختبار</h3>

<p>تجعل التبعيات الدائرية اختبار الوحدات صعباً للغاية. إذا كانت الوحدة A تعتمد على الوحدة B والوحدة B تعتمد على الوحدة A، فلا يمكنك اختبار أي منهما بمعزل دون محاكاة الأخرى. هذا يخلق وضعاً حيث إعداد اختبارك أكثر تعقيداً من الشيفرة المُختبرة، وقد لا تعكس محاكاتك بدقة السلوك الحقيقي للتبعية.</p>

<p>تأمل مثالاً حقيقياً من تطبيق Django. وحدة <code>orders</code> تستورد من <code>inventory</code> للتحقق من مستويات المخزون. وحدة <code>inventory</code> تستورد من <code>orders</code> لحساب الكميات المحجوزة. الآن، لاختبار وحدة <code>orders</code>، تحتاج لمحاكاة فحص المخزون. لكن المحاكاة تحتاج لفهم الكميات المحجوزة، وهذا يتطلب فهم الطلبات. ينتهي بك الأمر بتبعيات محاكاة دائرية تعكس تبعيات الشيفرة الدائرية، وتصبح اختباراتك هشة وبطيئة وغير موثوقة.</p>

<h3>اقتران النشر</h3>

<p>في بنية الخدمات المصغرة أو المونوليث المعياري، تمنع التبعيات الدائرية النشر المستقل. إذا كانت الخدمة A تعتمد على الخدمة B والخدمة B تعتمد على الخدمة A، فلا يمكنك نشر أي منهما بشكل مستقل. كل تغيير في أي من الخدمتين يتطلب نشراً منسقاً لكليهما، مما يلغي أحد الفوائد الأساسية للبنية المعيارية.</p>

<p>يمتد هذا الاقتران إلى حدود الفرق. إذا كان الفريق ألفا يملك الوحدة A والفريق بيتا يملك الوحدة B، فإن التبعية الدائرية تعني أنه لا يمكن لأي فريق الشحن دون التنسيق مع الآخر. تنخفض السرعة. يصبح تخطيط السبرنت مفاوضة. ويزداد الضغط "لإضافة استيراد آخر فقط" لأن الوحدات مقترنة بالفعل على أي حال.</p>

<h3>شلل إعادة الهيكلة</h3>

<p>ربما التكلفة الأكثر خبثاً هي أن التبعيات الدائرية تجعل إعادة الهيكلة تبدو مستحيلة. عندما تكون الوحدات مقترنة بإحكام في دورة، فإن تغيير واجهة وحدة واحدة يتطلب تغيير جميع الوحدات الأخرى في الدورة في وقت واحد. لا توجد طريقة للقيام بذلك تدريجياً. يؤدي هذا إلى عقلية إعادة هيكلة "الانفجار الكبير"، حيث تؤجل الفرق التحسينات الهيكلية لأن النطاق يبدو ساحقاً، وتزداد الدورة سوءاً مع كل سبرنت.</p>

<h2 id="real-world-examples">أمثلة من الواقع</h2>

<h3>دورة إعدادات Django</h3>

<p>نمط شائع في مشاريع Django هو دورة بين <code>settings</code> و <code>models</code> و <code>utils</code>. يستورد Settings من utils لحل المسارات. يستورد Utils من models لاستعلامات قاعدة البيانات. يستورد Models من settings لقيم التكوين. هذه الدورة الثلاثية تعني أن تغيير هيكل الإعدادات يتطلب لمس طبقة الأدوات المساعدة وربما طبقة النماذج، والعكس صحيح.</p>

<h3>حلقة Controller-Service في Node.js</h3>

<p>في تطبيقات Express.js، من الشائع رؤية المتحكمات تستورد الخدمات والخدمات تستورد المتحكمات (غالباً لمعالجة الأخطاء أو تنسيق الاستجابة). الحل مباشر: تقديم وحدة مشتركة للأخطاء/الاستجابة تعتمد عليها كلتا الطبقتين، مما يكسر الدورة. لكن بدون أداة لاكتشاف الدورة، غالباً لا تدرك الفرق وجودها حتى يحاولوا استخراج طبقة الخدمة في مكتبة مشتركة ويكتشفوا أنها لا يمكن أن تقف بمفردها.</p>

<h3>تشابك Event-Listener في Laravel</h3>

<p>تطور تطبيقات Laravel بشكل متكرر دورات بين فئات الأحداث ومستمعيها. حدث في فضاء الأسماء <code>Orders</code> يطلق مستمعاً في <code>Inventory</code>، الذي يرسل حدثاً مرة أخرى إلى <code>Orders</code>. بشكل فردي، كل فئة نظيفة. معاً، تشكل دورة وقت تشغيل يمكن أن تسبب حلقات لا نهائية في ظروف محددة وتجعل من المستحيل التفكير في تدفق الأحداث بدون تصور رسومي.</p>

<h2 id="how-to-detect-and-fix">كيفية اكتشاف وإصلاح التبعيات الدائرية</h2>

<h3>الاكتشاف باستخدام AigisCode</h3>

<p>الخطوة الأولى هي الرؤية. شغّل <code>aigiscode analyze /path/to/project</code> وافحص حقل <code>graph_analysis.strong_circular_dependencies</code> في تقرير JSON. كل إدخال يسرد الوحدات المتورطة في الدورة ومسارات الاستيراد التي تنشئها. يمنحك هذا خريطة دقيقة لمكان وجود الدورات وأي الاستيرادات تحتاج إلى إعادة هيكلة.</p>

<h3>إصلاح عكس التبعية</h3>

<p>الإصلاح الأكثر شيوعاً للتبعيات الدائرية هو <strong>عكس التبعية</strong>. بدلاً من أن تستورد الوحدة A مباشرة من الوحدة B والعكس، تقدم واجهة أو فئة أساسية مجردة تعتمد عليها كلتا الوحدتين. الوحدة A تعتمد على الواجهة. الوحدة B تنفذ الواجهة. سهم التبعية يشير الآن في اتجاه واحد فقط.</p>

<p>في Python، هذا يعني غالباً إنشاء وحدة <code>protocols.py</code> أو <code>interfaces.py</code> تحدد العقود بين الأنظمة الفرعية. في TypeScript، يعني استخراج الأنواع المشتركة في مجلد <code>types/</code> تستورد منه كلتا الوحدتين دون أن تستورد إحداهما من الأخرى.</p>

<h3>نمط الوسيط</h3>

<p>للدورات المدفوعة بالأحداث، نمط الوسيط فعال. بدلاً من أن تتواصل الوحدات مباشرة، تتواصل عبر ناقل أحداث مشترك أو وسيط. الوحدة A ترسل حدثاً. الوحدة B تستمع إليه. لا تستورد أي منهما من الأخرى. الوسيط هو التبعية المشتركة الوحيدة، ولا يحتوي على منطق أعمال، فقط توجيه.</p>

<h3>نمط الاستخراج والمشاركة</h3>

<p>أحياناً تكون الدورة موجودة لأن وحدتين تشتركان في مفهوم لم يُعطَ منزله الخاص. الإصلاح هو استخراج المفهوم المشترك في وحدة جديدة تعتمد عليها كلتا الوحدتين الأصليتين. على سبيل المثال، إذا كانت <code>orders.py</code> و <code>inventory.py</code> كلتاهما تحتاجان <code>ReservationCalculator</code>، استخرجه إلى <code>reservations.py</code> ودع كلتا الوحدتين تستوردان منه.</p>

<h2 id="prevention-is-cheaper-than-cure">الوقاية أرخص من العلاج</h2>

<p>أفضل نهج هو اكتشاف التبعيات الدائرية مبكراً ومنع تشكل جديدة. ادمج AigisCode في خط أنابيب CI الخاص بك. شغّله على كل طلب سحب. إذا ظهرت تبعية دائرية قوية جديدة، يجب وسم طلب السحب لمراجعة معمارية قبل الدمج.</p>

<p>تكلفة إصلاح تبعية دائرية تنمو أسياً مع عمر الدورة. دورة تُكتشف في طلب سحب تستغرق دقائق لإصلاحها. دورة نمت لمدة عامين قد تستغرق أسابيع من إعادة الهيكلة المنسقة. أدوات اكتشافها مبكراً موجودة اليوم. السؤال الوحيد هو هل يستخدمها فريقك.</p>
`,
      pl: `<h2 id="real-cost">Rzeczywisty koszt cyklicznych zależności</h2>
<p>Cykliczne zależności po cichu niszczą bazę kodu. AigisCode używa analizy Tarjan SCC do ich wykrywania.</p>`,
      bn: `
<p>প্রতিটি কোডবেসের একটি আকার আছে। আপনি হয়তো আপনার এডিটরে এটি দেখতে পান না, কিন্তু এটি সেখানে আছে: মডিউল, ক্লাস এবং ফাংশনগুলোকে সংযুক্ত করা ডিপেন্ডেন্সির একটি জাল। যখন সেই জালে লুপ থাকে, যেখানে মডিউল A মডিউল B-এর উপর নির্ভর করে এবং মডিউল B আবার মডিউল A-এর উপর নির্ভর করে, তখন আপনার একটি সার্কুলার ডিপেন্ডেন্সি আছে। এবং যদিও এটি একটি ছোটখাটো স্ট্রাকচারাল বৈশিষ্ট্য মনে হতে পারে, সার্কুলার ডিপেন্ডেন্সি হলো টেকনিক্যাল ডেটের সবচেয়ে ব্যয়বহুল রূপগুলোর মধ্যে একটি যা একটি কোডবেস জমা করতে পারে।</p>

<h2 id="what-circular-dependencies-are">সার্কুলার ডিপেন্ডেন্সি আসলে কী</h2>

<p>একটি সার্কুলার ডিপেন্ডেন্সি তখন বিদ্যমান থাকে যখন দুই বা ততোধিক মডিউল একটি ডিপেন্ডেন্সি সাইকেল গঠন করে। সবচেয়ে সরল ক্ষেত্রে একটি সরাসরি সাইকেল: <code>auth.py</code> <code>users.py</code> থেকে ইমপোর্ট করে, এবং <code>users.py</code> <code>auth.py</code> থেকে ইমপোর্ট করে। কিন্তু বাস্তব-বিশ্বের সাইকেলগুলো প্রায়ই দীর্ঘ এবং খুঁজে পাওয়া কঠিন। একটি সাইকেলে চার বা পাঁচটি মডিউল জড়িত থাকতে পারে, প্রতিটি পৃথকভাবে পরিষ্কার দেখায়, কিন্তু একসাথে একটি লুপ গঠন করে যা তাদের একটি অবিচ্ছেদ্য ইউনিটে বেঁধে ফেলে।</p>

<p>দুই ধরনের সাইকেলের মধ্যে পার্থক্য করা গুরুত্বপূর্ণ। <strong>শক্তিশালী সার্কুলার ডিপেন্ডেন্সি</strong> হলো আর্কিটেকচারাল সাইকেল। এগুলো মডিউল বা প্যাকেজ স্তরে বিদ্যমান এবং নির্দেশ করে যে দুটি সাবসিস্টেম মৌলিকভাবে জড়িয়ে পড়েছে। <strong>সম্পূর্ণ সার্কুলার ডিপেন্ডেন্সি</strong>-তে রানটাইম এবং লোড-অর্ডার সাইকেল অন্তর্ভুক্ত যা lazy import, conditional require, বা ফ্রেমওয়ার্ক ম্যাজিকের কারণে থাকতে পারে। উভয়ই জানার যোগ্য, কিন্তু শক্তিশালী সাইকেলগুলোই সবচেয়ে বেশি ক্ষতি করে।</p>

<p>AigisCode তার বিশ্লেষণে এই পার্থক্য স্পষ্টভাবে তুলে ধরে। JSON রিপোর্ট <code>strong_circular_dependencies</code> এবং <code>circular_dependencies</code> আলাদা করে, যা টিমদের সত্যিকারের রিফ্যাক্টরিং প্রয়োজন এমন আর্কিটেকচারাল সাইকেলগুলোকে অগ্রাধিকার দিতে দেয় এবং সচেতনতার জন্য রানটাইম সাইকেলগুলো নোট করে।</p>

<h2 id="why-they-are-dangerous">সার্কুলার ডিপেন্ডেন্সি কেন বিপজ্জনক</h2>

<h3>কম্পাইলেশন এবং লোড-অর্ডার ব্যর্থতা</h3>

<p>কঠোর মডিউল রেজোলিউশন সহ ভাষাগুলোতে, সার্কুলার ডিপেন্ডেন্সি সরাসরি ব্যর্থতা ঘটাতে পারে। Python-এর ইমপোর্ট সিস্টেম একটি সাইকেলের সম্মুখীন হলে একটি মডিউল আংশিকভাবে এক্সিকিউট করবে, যা রানটাইমে <code>ImportError</code> বা <code>AttributeError</code> ঘটায় যখন একটি নাম এখনও সংজ্ঞায়িত হয়নি। কঠোর ES module সেমান্টিকস সহ TypeScript-এ, সার্কুলার ইমপোর্ট ব্যবহারের সময়ে <code>undefined</code> মান দিতে পারে কারণ মডিউলটি ইনিশিয়ালাইজ হওয়া শেষ হয়নি। অটোলোডিং সহ PHP-তে, সার্কুলার ডিপেন্ডেন্সি সূক্ষ্ম বাগ তৈরি করতে পারে যেখানে একটি ক্লাস উপলব্ধ মনে হয় কিন্তু তার ডিপেন্ডেন্সিগুলো এখনও লোড হয়নি।</p>

<p>এই ব্যর্থতাগুলো ডিবাগ করা অত্যন্ত কঠিন কারণ এগুলো <em>ইমপোর্ট অর্ডার</em>-এর উপর নির্ভর করে, যা কোন এন্ট্রি পয়েন্ট কোড পাথ ট্রিগার করেছে তার উপর ভিত্তি করে পরিবর্তিত হয়। একই টেস্ট স্যুট কোন টেস্ট আগে চলে তার উপর নির্ভর করে পাস বা ফেল করতে পারে।</p>

<h3>টেস্টিং দুঃস্বপ্ন</h3>

<p>সার্কুলার ডিপেন্ডেন্সি ইউনিট টেস্টিংকে অসাধারণভাবে কঠিন করে তোলে। যদি মডিউল A মডিউল B-এর উপর নির্ভর করে এবং মডিউল B মডিউল A-এর উপর নির্ভর করে, তাহলে আপনি অন্যটিকে মক না করে কোনোটিকেই আলাদাভাবে টেস্ট করতে পারবেন না। এটি এমন একটি পরিস্থিতি তৈরি করে যেখানে আপনার টেস্ট সেটআপ টেস্ট করা কোডের চেয়ে বেশি জটিল, এবং আপনার মকগুলো ডিপেন্ডেন্সির প্রকৃত আচরণ সঠিকভাবে প্রতিফলিত নাও করতে পারে।</p>

<p>একটি Django অ্যাপ্লিকেশনের বাস্তব উদাহরণ বিবেচনা করুন। <code>orders</code> মডিউল স্টক লেভেল চেক করতে <code>inventory</code> থেকে ইমপোর্ট করে। <code>inventory</code> মডিউল রিজার্ভড পরিমাণ গণনা করতে <code>orders</code> থেকে ইমপোর্ট করে। এখন, <code>orders</code> মডিউল ইউনিট টেস্ট করতে, আপনাকে ইনভেন্টরি চেক মক করতে হবে। কিন্তু মকটিকে রিজার্ভড পরিমাণ বুঝতে হবে, যার জন্য অর্ডার বোঝা প্রয়োজন। আপনি সার্কুলার মক ডিপেন্ডেন্সিতে পড়ে যান যা সার্কুলার কোড ডিপেন্ডেন্সিকে প্রতিফলিত করে, এবং আপনার টেস্টগুলো ভঙ্গুর, ধীর এবং অনির্ভরযোগ্য হয়ে যায়।</p>

<h3>ডিপ্লয়মেন্ট কাপলিং</h3>

<p>একটি মাইক্রোসার্ভিস বা মডিউলার মনোলিথ আর্কিটেকচারে, সার্কুলার ডিপেন্ডেন্সি স্বাধীন ডিপ্লয়মেন্ট প্রতিরোধ করে। যদি সার্ভিস A সার্ভিস B-এর উপর নির্ভর করে এবং সার্ভিস B সার্ভিস A-এর উপর নির্ভর করে, তাহলে আপনি কোনোটিকেই স্বাধীনভাবে ডিপ্লয় করতে পারবেন না। যেকোনো সার্ভিসে প্রতিটি পরিবর্তনের জন্য উভয়ের সমন্বিত ডিপ্লয়মেন্ট প্রয়োজন, মডিউলার আর্কিটেকচারের প্রাথমিক সুবিধাগুলোর একটি দূর করে।</p>

<p>এই কাপলিং টিম সীমানায় প্রসারিত হয়। যদি টিম Alpha মডিউল A-এর মালিক হয় এবং টিম Beta মডিউল B-এর মালিক হয়, একটি সার্কুলার ডিপেন্ডেন্সি মানে কোনো টিমই অন্যের সাথে সমন্বয় ছাড়া শিপ করতে পারে না। বেগ কমে যায়। স্প্রিন্ট প্ল্যানিং একটি আলোচনায় পরিণত হয়। এবং "শুধু আরেকটি ইমপোর্ট যোগ করা"-এর চাপ বাড়ে কারণ মডিউলগুলো ইতিমধ্যেই কাপল্ড।</p>

<h3>রিফ্যাক্টরিং পক্ষাঘাত</h3>

<p>সম্ভবত সবচেয়ে কপট খরচ হলো যে সার্কুলার ডিপেন্ডেন্সি রিফ্যাক্টরিংকে অসম্ভব মনে করায়। যখন মডিউলগুলো একটি সাইকেলে শক্তভাবে কাপল্ড থাকে, একটি মডিউলের ইন্টারফেস পরিবর্তন করতে একই সময়ে সাইকেলের অন্য সবগুলো পরিবর্তন করতে হয়। ক্রমবর্ধমানভাবে এটি করার কোনো উপায় নেই। এটি একটি "বিগ ব্যাং" রিফ্যাক্টরিং মানসিকতার দিকে নিয়ে যায়, যেখানে টিমগুলো স্ট্রাকচারাল উন্নতি স্থগিত করে কারণ পরিসর অপ্রতিরোধ্য মনে হয়, এবং প্রতিটি স্প্রিন্টে সাইকেল আরও খারাপ হয়।</p>

<h2 id="real-world-examples">বাস্তব-বিশ্বের উদাহরণ</h2>

<h3>Django Settings সাইকেল</h3>

<p>Django প্রজেক্টে একটি সাধারণ প্যাটার্ন হলো <code>settings</code>, <code>models</code>, এবং <code>utils</code>-এর মধ্যে একটি সাইকেল। Settings পাথ রেজোলিউশনের জন্য utils থেকে ইমপোর্ট করে। Utils ডেটাবেস কোয়েরির জন্য models থেকে ইমপোর্ট করে। Models কনফিগারেশন মানের জন্য settings থেকে ইমপোর্ট করে। এই তিন-মুখী সাইকেল মানে settings স্ট্রাকচার পরিবর্তন করতে ইউটিলিটি লেয়ার এবং সম্ভাব্যভাবে মডেল লেয়ার স্পর্শ করতে হবে, এবং বিপরীতভাবেও।</p>

<h3>Node.js Controller-Service লুপ</h3>

<p>Express.js অ্যাপ্লিকেশনে, কন্ট্রোলারগুলো সার্ভিস ইমপোর্ট করে এবং সার্ভিসগুলো কন্ট্রোলার ইমপোর্ট করে (প্রায়ই এরর হ্যান্ডলিং বা রেসপন্স ফরম্যাটিংয়ের জন্য) দেখা সাধারণ। সমাধান সরল: একটি শেয়ার্ড এরর/রেসপন্স মডিউল তৈরি করুন যার উপর উভয় লেয়ার নির্ভর করে, সাইকেল ভাঙে। কিন্তু সাইকেল শনাক্ত করার টুল ছাড়া, টিমগুলো প্রায়ই বুঝতে পারে না যে এটি বিদ্যমান যতক্ষণ না তারা সার্ভিস লেয়ারকে একটি শেয়ার্ড লাইব্রেরিতে এক্সট্র্যাক্ট করতে চায় এবং আবিষ্কার করে যে এটি একা দাঁড়াতে পারে না।</p>

<h3>Laravel Event-Listener জট</h3>

<p>Laravel অ্যাপ্লিকেশনগুলো ইভেন্ট ক্লাস এবং তাদের লিসেনারদের মধ্যে প্রায়ই সাইকেল তৈরি করে। <code>Orders</code> নেমস্পেসে একটি ইভেন্ট <code>Inventory</code>-তে একটি লিসেনার ট্রিগার করে, যা <code>Orders</code>-এ আবার একটি ইভেন্ট ডিসপ্যাচ করে। পৃথকভাবে, প্রতিটি ক্লাস পরিষ্কার। একসাথে, তারা একটি রানটাইম সাইকেল গঠন করে যা নির্দিষ্ট পরিস্থিতিতে অসীম লুপ তৈরি করতে পারে এবং গ্রাফ ভিজ্যুয়ালাইজেশন ছাড়া ইভেন্ট ফ্লো সম্পর্কে যুক্তি করা অসম্ভব করে তোলে।</p>

<h2 id="how-to-detect-and-fix">সার্কুলার ডিপেন্ডেন্সি কিভাবে শনাক্ত এবং ঠিক করবেন</h2>

<h3>AigisCode দিয়ে শনাক্তকরণ</h3>

<p>প্রথম পদক্ষেপ হলো দৃশ্যমানতা। <code>aigiscode analyze /path/to/project</code> চালান এবং JSON রিপোর্টে <code>graph_analysis.strong_circular_dependencies</code> ফিল্ড পরীক্ষা করুন। প্রতিটি এন্ট্রি সাইকেলে জড়িত মডিউলগুলো এবং এটি তৈরি করা ইমপোর্ট পাথগুলো তালিকাভুক্ত করে। এটি আপনাকে সাইকেলগুলো কোথায় এবং কোন ইমপোর্টগুলো পুনর্গঠন করা দরকার তার একটি সুনির্দিষ্ট মানচিত্র দেয়।</p>

<h3>Dependency Inversion সমাধান</h3>

<p>সার্কুলার ডিপেন্ডেন্সির সবচেয়ে সাধারণ সমাধান হলো <strong>dependency inversion</strong>। মডিউল A সরাসরি মডিউল B থেকে ইমপোর্ট করার পরিবর্তে এবং বিপরীতভাবে, আপনি একটি ইন্টারফেস বা অ্যাবস্ট্রাক্ট বেস ক্লাস প্রবর্তন করেন যার উপর উভয় মডিউল নির্ভর করে। মডিউল A ইন্টারফেসের উপর নির্ভর করে। মডিউল B ইন্টারফেস ইমপ্লিমেন্ট করে। ডিপেন্ডেন্সি তীর এখন শুধু একটি দিকে নির্দেশ করে।</p>

<p>Python-এ, এর মানে প্রায়ই একটি <code>protocols.py</code> বা <code>interfaces.py</code> মডিউল তৈরি করা যা সাবসিস্টেমগুলোর মধ্যে কন্ট্র্যাক্ট সংজ্ঞায়িত করে। TypeScript-এ, এর মানে শেয়ার্ড টাইপগুলো একটি <code>types/</code> ডিরেক্টরিতে এক্সট্র্যাক্ট করা যেখান থেকে উভয় মডিউল একে অপরের থেকে ইমপোর্ট না করেই ইমপোর্ট করে।</p>

<h3>Mediator প্যাটার্ন</h3>

<p>ইভেন্ট-চালিত সাইকেলের জন্য, mediator প্যাটার্ন কার্যকর। মডিউলগুলো সরাসরি যোগাযোগ করার পরিবর্তে, একটি শেয়ার্ড ইভেন্ট বাস বা mediator-এর মাধ্যমে যোগাযোগ করে। মডিউল A একটি ইভেন্ট ডিসপ্যাচ করে। মডিউল B এটি শোনে। কেউ অন্যের থেকে ইমপোর্ট করে না। Mediator হলো একমাত্র শেয়ার্ড ডিপেন্ডেন্সি, এবং এতে কোনো ব্যবসায়িক যুক্তি নেই, শুধু রাউটিং।</p>

<h3>Extract-and-Share প্যাটার্ন</h3>

<p>কখনো কখনো সাইকেল বিদ্যমান থাকে কারণ দুটি মডিউল এমন একটি ধারণা ভাগ করে যাকে এখনও তার নিজস্ব জায়গা দেওয়া হয়নি। সমাধান হলো শেয়ার্ড ধারণাটিকে একটি নতুন মডিউলে এক্সট্র্যাক্ট করা যার উপর উভয় মূল মডিউল নির্ভর করে। উদাহরণস্বরূপ, যদি <code>orders.py</code> এবং <code>inventory.py</code> উভয়ের একটি <code>ReservationCalculator</code> প্রয়োজন হয়, এটিকে <code>reservations.py</code>-তে এক্সট্র্যাক্ট করুন এবং উভয় মডিউলকে এটি থেকে ইমপোর্ট করতে দিন।</p>

<h2 id="prevention-is-cheaper-than-cure">প্রতিরোধ নিরাময়ের চেয়ে সস্তা</h2>

<p>সর্বোত্তম পদ্ধতি হলো সার্কুলার ডিপেন্ডেন্সি তাড়াতাড়ি শনাক্ত করা এবং নতুনগুলো গঠন হওয়া প্রতিরোধ করা। আপনার CI পাইপলাইনে AigisCode ইন্টিগ্রেট করুন। প্রতিটি পুল রিকোয়েস্টে এটি চালান। যদি একটি নতুন শক্তিশালী সার্কুলার ডিপেন্ডেন্সি দেখা দেয়, মার্জ করার আগে PR-টি আর্কিটেকচারাল রিভিউয়ের জন্য ফ্ল্যাগ করা উচিত।</p>

<p>একটি সার্কুলার ডিপেন্ডেন্সি ঠিক করার খরচ সাইকেলের বয়সের সাথে সাথে তীব্রভাবে বৃদ্ধি পায়। একটি PR-এ ধরা পড়া সাইকেল ঠিক করতে কয়েক মিনিট লাগে। দুই বছর ধরে বেড়ে চলা একটি সাইকেল ঠিক করতে সমন্বিত রিফ্যাক্টরিংয়ে কয়েক সপ্তাহ লাগতে পারে। তাড়াতাড়ি ধরার টুলগুলো আজই বিদ্যমান। একমাত্র প্রশ্ন হলো আপনার টিম সেগুলো ব্যবহার করে কিনা।</p>
`,
    },
  },

  /* ======================================================================== */
  /*  3. Dead Code: The Silent Growth of Technical Debt                       */
  /* ======================================================================== */
  {
    slug: 'dead-code-technical-debt',
    date: '2026-01-28',
    readTime: 8,
    tags: ['Dead Code', 'Technical Debt', 'Maintenance'],
    image: '/blog-dead-code.jpg',
    author: { name: 'David Strejc', role: 'Creator of AigisCode' },
    relatedSlugs: [
      'circular-dependencies-real-cost',
      'why-ai-code-analysis-matters-2026',
    ],
    title: {
      en: 'Dead Code: The Silent Growth of Technical Debt',
      cs: 'Mrtvý kód: Tichý růst technického dluhu',
      fr: 'Code mort : la croissance silencieuse de la dette technique',
      es: 'Codigo muerto: el crecimiento silencioso de la deuda tecnica',
      zh: '死代码：技术债务的无声增长',
      hi: 'डेड कोड: तकनीकी ऋण की मूक वृद्धि',
      pt: 'Código morto: o crescimento silencioso da dívida técnica',
      ar: 'الشيفرة الميتة: النمو الصامت للديون التقنية',
      pl: 'Martwy kod: cichy wzrost długu technicznego',
      bn: 'ডেড কোড: টেকনিক্যাল ডেটের নীরব বৃদ্ধি',
    },
    description: {
      en: 'Dead code accumulates silently in every codebase. Learn about the real costs of unused imports, abandoned classes, and orphan files, plus practical strategies for detection and cleanup.',
      cs: 'Mrtvý kód se tiše hromadí. Poznejte skutečné náklady a strategie pro jeho detekci a čištění.',
      fr: 'Le code mort s\'accumule silencieusement. Decouvrez les couts reels et les strategies de nettoyage.',
      es: 'El codigo muerto se acumula silenciosamente. Conozca los costos reales y las estrategias de limpieza.',
      zh: '死代码在每个代码库中默默积累。了解实际成本和清理策略。',
      hi: 'डेड कोड हर कोडबेस में चुपचाप जमा होता है। वास्तविक लागत और सफाई रणनीतियों के बारे में जानें।',
      pt: 'Código morto se acumula silenciosamente. Conheça os custos reais e as estratégias de limpeza.',
      ar: 'تتراكم الشيفرة الميتة بصمت في كل قاعدة شيفرة. تعرّف على التكاليف الحقيقية للاستيرادات غير المستخدمة والفئات المهجورة والملفات اليتيمة بالإضافة إلى استراتيجيات عملية للاكتشاف والتنظيف.',
      pl: 'Martwy kod gromadzi się po cichu w każdej bazie kodu. Poznaj rzeczywiste koszty nieużywanych importów, porzuconych klas i osieroconych plików — oraz jak systematycznie je usuwać.',
      bn: 'ডেড কোড প্রতিটি কোডবেসে নীরবে জমা হয়। অব্যবহৃত ইমপোর্ট, পরিত্যক্ত ক্লাস এবং অনাথ ফাইলের প্রকৃত খরচ এবং শনাক্তকরণ ও পরিচ্ছন্নতার ব্যবহারিক কৌশল শিখুন।',
    },
    metaDescription: {
      en: 'Dead code silently grows technical debt. Understand the real costs of unused imports, abandoned classes, and orphan files. Learn detection and cleanup strategies with AigisCode.',
      cs: 'Mrtvý kód tiše zvyšuje technický dluh. Poznejte strategie detekce a čištění s AigisCode.',
      fr: 'Le code mort augmente silencieusement la dette technique. Apprenez les strategies de detection et nettoyage avec AigisCode.',
      es: 'El codigo muerto aumenta silenciosamente la deuda tecnica. Aprenda estrategias de deteccion con AigisCode.',
      zh: '死代码默默增加技术债务。了解使用 AigisCode 的检测和清理策略。',
      hi: 'डेड कोड चुपचाप तकनीकी ऋण बढ़ाता है। AigisCode के साथ पहचान और सफाई रणनीतियाँ सीखें।',
      pt: 'Código morto aumenta silenciosamente a dívida técnica. Aprenda estratégias de detecção com o AigisCode.',
      ar: 'تنمّي الشيفرة الميتة الديون التقنية بصمت. افهم التكاليف الحقيقية للاستيرادات غير المستخدمة والفئات المهجورة والملفات اليتيمة. تعلّم استراتيجيات الاكتشاف والتنظيف مع AigisCode.',
      pl: 'Martwy kod po cichu zwiększa dług techniczny. Poznaj rzeczywiste koszty nieużywanych importów, porzuconych klas i osieroconych plików. Poznaj strategie detekcji z AigisCode.',
      bn: 'ডেড কোড নীরবে টেকনিক্যাল ডেট বাড়ায়। অব্যবহৃত ইমপোর্ট, পরিত্যক্ত ক্লাস এবং অনাথ ফাইলের প্রকৃত খরচ বুঝুন। AigisCode দিয়ে শনাক্তকরণ ও পরিচ্ছন্নতার কৌশল শিখুন।',
    },
    content: {
      en: `
<p>Dead code is the sediment of software development. Every feature that gets replaced, every experiment that does not ship, every refactoring that moves logic to a new location but forgets to remove the old one, they all leave behind code that is no longer executed. And unlike a broken test or a compile error, dead code does not announce its presence. It just sits there, quietly increasing the size, complexity, and maintenance burden of your codebase.</p>

<h2 id="how-dead-code-accumulates">How Dead Code Accumulates</h2>

<p>Dead code rarely appears through deliberate action. No developer writes a function with the intent of never calling it. Instead, dead code emerges through the natural lifecycle of software development.</p>

<p><strong>Feature replacement.</strong> A team builds version 2 of a feature, carefully routes all traffic to the new implementation, but never deletes the version 1 code because "we might need to roll back." Six months later, the rollback window has passed, but the old code remains because nobody remembers which files are safe to delete.</p>

<p><strong>Refactoring residue.</strong> A developer moves a utility function from <code>helpers.py</code> to <code>utils/string_helpers.py</code>. All call sites are updated. But the original function definition in <code>helpers.py</code> is left in place because the developer was not certain nothing else referenced it. The IDE shows no direct callers, but what about dynamic imports? What about tests? The safer choice feels like leaving it alone.</p>

<p><strong>Copy-paste evolution.</strong> A method is copied from one class to another to make a quick fix. The original is never removed. Over time, both copies evolve independently, and developers are unsure which one is the "real" version.</p>

<p><strong>Configuration drift.</strong> Environment variables, feature flags, and config keys accumulate over multiple release cycles. Old flags for features that shipped months ago remain in the config files, and the code that checks them remains in the application, even though the flag is always set to the same value.</p>

<h2 id="the-real-costs">The Real Costs of Dead Code</h2>

<h3>Developer Confusion</h3>

<p>The most immediate cost of dead code is cognitive load. A new developer joins the team, opens the codebase, and encounters a module with 15 classes. Five of them are dead. But the new developer does not know that. They spend time reading, understanding, and forming mental models of code that does not matter. Worse, they might build new features on top of dead code, creating dependencies on zombie implementations that nobody maintains.</p>

<p>This is not a theoretical concern. A 2025 survey of 1,200 developers by JetBrains found that <strong>63% of respondents</strong> cited "understanding unfamiliar code" as their biggest productivity bottleneck. Dead code directly inflates the amount of unfamiliar code that developers must navigate.</p>

<h3>Larger Bundles and Slower Builds</h3>

<p>In frontend applications, dead code directly impacts user experience. Unused components, abandoned utility functions, and orphan modules all get bundled into production JavaScript if tree-shaking fails to eliminate them. And tree-shaking has limits. If a module has side effects, or if the dead code is referenced through dynamic imports that the bundler cannot statically analyze, it ends up in the bundle.</p>

<p>On the backend, dead code increases build times, test execution time, and container image sizes. Every dead file is another file that the test runner must discover and skip. Every dead import is another module that must be resolved during startup.</p>

<h3>Expanded Security Surface</h3>

<p>Dead code that is still present in the repository is still present in production. If that dead code contains a vulnerability, such as an old API client that uses an insecure authentication method, or a deprecated utility that performs unsanitized string interpolation, the vulnerability is exploitable even if no active code path reaches it. An attacker who gains the ability to call arbitrary functions does not care whether those functions are "dead" from the application's perspective.</p>

<h3>CI and Tooling Overhead</h3>

<p>Every dead file adds to the workload of your continuous integration pipeline. Linters analyze it. Type checkers process it. Code coverage tools report on it, showing artificially low coverage percentages because dead code has zero test coverage by definition. Security scanners flag vulnerabilities in it. And developers spend time triaging these findings, only to discover they are about code that nobody uses.</p>

<h2 id="types-of-dead-code">Types of Dead Code</h2>

<p>Dead code is not monolithic. AigisCode detects several distinct categories, each with different detection strategies and risk profiles.</p>

<p><strong>Unused imports</strong> are the most common and easiest to detect. A module imports a symbol that is never used in the rest of the file. Most linters catch these at the single-file level, but cross-module unused imports, where a module re-exports a symbol that nobody else imports, require codebase-wide analysis.</p>

<p><strong>Unreferenced methods</strong> are class methods that are defined but never called from anywhere in the codebase. These are harder to detect because methods might be called dynamically through reflection, decorators, or framework conventions. AigisCode uses confidence levels to distinguish between methods that are definitively unused and those that might be called through dynamic dispatch.</p>

<p><strong>Abandoned classes</strong> are entire classes that are never instantiated or referenced. These often result from feature replacement where the new implementation uses a different class name or architecture.</p>

<p><strong>Orphan files</strong> are files with no inbound dependencies. No other file imports from them, no configuration references them, and no test targets them. These are often the clearest indicators of dead code because a file with no inbound dependencies can be safely deleted without affecting any other part of the codebase.</p>

<p><strong>Orphan properties</strong> are class attributes or object properties that are assigned but never read. These accumulate as data models evolve and fields are added for features that are later abandoned.</p>

<h2 id="detection-strategies">Detection Strategies</h2>

<p>Detecting dead code reliably requires cross-file analysis. A function that appears unused in its own file might be the only export of a utility module that 30 other files depend on. Conversely, a function that is imported in another file might only be imported for type-checking purposes and never actually called at runtime.</p>

<p>AigisCode approaches this by building a complete dependency graph from tree-sitter AST analysis. Every import, every function call, every class instantiation is tracked across the entire codebase. The dead code detector then identifies symbols that have no inbound references in the graph, flagging them with confidence levels based on how certain the analysis is.</p>

<p>For frameworks with convention-based code discovery, like Django's view functions referenced in URL configs or Laravel's service providers registered in config files, AigisCode's policy system allows teams to mark these entry points explicitly, preventing false positives without disabling the detector entirely.</p>

<h2 id="practical-cleanup-workflow">A Practical Cleanup Workflow</h2>

<p>Cleaning up dead code should be systematic, not heroic. Here is a workflow that teams can follow.</p>

<p><strong>Step 1: Baseline scan.</strong> Run <code>aigiscode analyze /path/to/project</code> to generate the initial report. Review the <code>dead_code</code> section of the JSON output.</p>

<p><strong>Step 2: Triage by confidence.</strong> Start with high-confidence findings. These are symbols that the deterministic analysis is certain are unused. Orphan files and unused imports with no dynamic reference patterns are typically high confidence.</p>

<p><strong>Step 3: Sample and verify.</strong> Before bulk-deleting, manually verify a sample of findings. Check that the code is truly unreachable. Search for dynamic references, configuration-based discovery, and reflection patterns that the static analysis might have missed.</p>

<p><strong>Step 4: Delete in small batches.</strong> Remove dead code in focused pull requests, one category or module at a time. This makes code review manageable and allows for easy reversion if a finding turns out to be a false positive.</p>

<p><strong>Step 5: Update policy.</strong> For false positives, add exclusion rules to <code>.aigiscode/rules.json</code> so they do not reappear in future scans. For patterns of false positives, encode them in <code>.aigiscode/policy.json</code> so the detector learns to skip similar patterns.</p>

<p><strong>Step 6: Prevent regression.</strong> Add AigisCode to your CI pipeline. Flag new dead code introductions in pull requests. The goal is not zero dead code but a consistent trend toward less of it.</p>

<h2 id="the-bottom-line">The Bottom Line</h2>

<p>Dead code is not an emergency. It does not cause production outages or data loss. But it is a slow, compounding tax on every engineering activity. It slows onboarding, inflates bundles, confuses developers, expands the security surface, and makes every other form of technical debt harder to address. The good news is that detecting it is a solved problem. The tools exist. The workflow is straightforward. The only barrier is making the decision to start.</p>
`,
      cs: `
<p>Mrtvý kód je sedimentem vývoje softwaru. Každá funkce, která je nahrazena, každý experiment, který se nedodá, každý refaktoring, který přesune logiku na nové místo, ale zapomene odstranit staré — to vše za sebou zanechává kód, který se už nevykonává. A na rozdíl od spadlého testu nebo chyby kompilace mrtvý kód svou přítomnost neoznamuje. Jen tam sedí a tiše zvyšuje velikost, složitost a údržbovou zátěž vašeho codebase.</p>

<h2 id="how-dead-code-accumulates">Jak se mrtvý kód hromadí</h2>

<p>Mrtvý kód zřídka vzniká záměrným jednáním. Žádný vývojář nepíše funkci s úmyslem ji nikdy nevolat. Místo toho mrtvý kód vzniká přirozeným životním cyklem vývoje softwaru.</p>

<p><strong>Nahrazení funkce.</strong> Tým vytvoří verzi 2 funkce, pečlivě přesměruje veškerý provoz na novou implementaci, ale nikdy nesmaže kód verze 1, protože „možná budeme potřebovat rollback." O šest měsíců později okno pro rollback uplynulo, ale starý kód zůstává, protože si nikdo nepamatuje, které soubory je bezpečné smazat.</p>

<p><strong>Rezidua refaktoringu.</strong> Vývojář přesune utilitní funkci z <code>helpers.py</code> do <code>utils/string_helpers.py</code>. Všechna místa volání jsou aktualizována. Ale původní definice funkce v <code>helpers.py</code> zůstává na místě, protože si vývojář nebyl jistý, že ji nic jiného neodkazuje. IDE neukazuje žádné přímé volající, ale co dynamické importy? Co testy? Bezpečnější volbou se zdá nechat to být.</p>

<p><strong>Evoluce copy-paste.</strong> Metoda je zkopírována z jedné třídy do druhé pro rychlou opravu. Originál se nikdy neodstraní. Časem se obě kopie vyvíjejí nezávisle a vývojáři si nejsou jisti, která je ta „skutečná" verze.</p>

<p><strong>Drift konfigurace.</strong> Proměnné prostředí, feature flagy a konfigurační klíče se hromadí přes více release cyklů. Staré flagy pro funkce dodané před měsíci zůstávají v konfiguračních souborech a kód, který je kontroluje, zůstává v aplikaci, i když je flag vždy nastaven na stejnou hodnotu.</p>

<h2 id="the-real-costs">Skutečné náklady mrtvého kódu</h2>

<h3>Zmatení vývojářů</h3>

<p>Nejbezprostřednějším nákladem mrtvého kódu je kognitivní zátěž. Nový vývojář se připojí k týmu, otevře codebase a narazí na modul s 15 třídami. Pět z nich je mrtvých. Ale nový vývojář to neví. Tráví čas čtením, porozuměním a tvorbou mentálních modelů kódu, na kterém nezáleží. Co je horší, může stavět nové funkce na mrtvém kódu a vytvářet závislosti na zombie implementacích, které nikdo neudržuje.</p>

<p>Toto není teoretická obava. Průzkum 1 200 vývojářů od JetBrains z roku 2025 zjistil, že <strong>63 % respondentů</strong> uvedlo „porozumění neznámému kódu" jako svou největší překážku produktivity. Mrtvý kód přímo nafukuje množství neznámého kódu, kterým musí vývojáři procházet.</p>

<h3>Větší bundly a pomalejší sestavení</h3>

<p>Ve frontendových aplikacích mrtvý kód přímo ovlivňuje uživatelský zážitek. Nepoužívané komponenty, opuštěné utilitní funkce a osiřelé moduly jsou všechny zabaleny do produkčního JavaScriptu, pokud je tree-shaking nedokáže eliminovat. A tree-shaking má své limity. Pokud má modul vedlejší efekty, nebo pokud je mrtvý kód odkazován přes dynamické importy, které bundler nedokáže staticky analyzovat, skončí v bundlu.</p>

<p>Na backendu mrtvý kód zvyšuje dobu sestavení, dobu spouštění testů a velikost kontejnerových obrazů. Každý mrtvý soubor je dalším souborem, který musí test runner objevit a přeskočit. Každý mrtvý import je dalším modulem, který musí být vyřešen při startu.</p>

<h3>Rozšířená bezpečnostní plocha</h3>

<p>Mrtvý kód, který je stále přítomen v repozitáři, je stále přítomen v produkci. Pokud tento mrtvý kód obsahuje zranitelnost — například starého API klienta používajícího nezabezpečenou autentizační metodu nebo zastaralou utilitu provádějící nesanitizovanou interpolaci řetězců — zranitelnost je zneužitelná, i když k ní žádná aktivní cesta kódu nevede. Útočníkovi, který získá schopnost volat libovolné funkce, je jedno, zda jsou tyto funkce z pohledu aplikace „mrtvé".</p>

<h3>Režie CI a nástrojů</h3>

<p>Každý mrtvý soubor přidává práci vaší CI pipeline. Lintery jej analyzují. Typové kontroly jej zpracovávají. Nástroje pro pokrytí kódu o něm reportují, zobrazují uměle nízká procenta pokrytí, protože mrtvý kód má z definice nulové pokrytí testy. Bezpečnostní skenery v něm označují zranitelnosti. A vývojáři tráví čas tříděním těchto nálezů, aby zjistili, že se týkají kódu, který nikdo nepoužívá.</p>

<h2 id="types-of-dead-code">Typy mrtvého kódu</h2>

<p>Mrtvý kód není monolitický. AigisCode detekuje několik odlišných kategorií, každou s jinými strategiemi detekce a rizikovými profily.</p>

<p><strong>Nepoužívané importy</strong> jsou nejběžnější a nejsnáze detekovatelné. Modul importuje symbol, který se nikdy nepoužije ve zbytku souboru. Většina linterů je zachytí na úrovni jednoho souboru, ale cross-modulové nepoužívané importy — kde modul re-exportuje symbol, který nikdo jiný neimportuje — vyžadují analýzu celého codebase.</p>

<p><strong>Neodkazované metody</strong> jsou metody tříd, které jsou definovány, ale nikdy volány odkudkoli v codebase. Jsou obtížněji detekovatelné, protože metody mohou být volány dynamicky přes reflexi, dekorátory nebo konvence frameworku. AigisCode používá úrovně spolehlivosti k rozlišení mezi metodami, které jsou definitivně nepoužívané, a těmi, které mohou být volány přes dynamický dispatch.</p>

<p><strong>Opuštěné třídy</strong> jsou celé třídy, které nejsou nikdy instanciovány ani odkazovány. Často vznikají z nahrazení funkce, kde nová implementace používá jiný název třídy nebo architekturu.</p>

<p><strong>Osiřelé soubory</strong> jsou soubory bez příchozích závislostí. Žádný jiný soubor z nich neimportuje, žádná konfigurace na ně neodkazuje a žádný test na ně necílí. Jsou často nejjasnějšími indikátory mrtvého kódu, protože soubor bez příchozích závislostí může být bezpečně smazán bez ovlivnění jakékoli jiné části codebase.</p>

<p><strong>Osiřelé vlastnosti</strong> jsou atributy tříd nebo vlastnosti objektů, které jsou přiřazeny, ale nikdy čteny. Hromadí se s vývojem datových modelů a přidáváním polí pro funkce, které jsou později opuštěny.</p>

<h2 id="detection-strategies">Strategie detekce</h2>

<p>Spolehlivá detekce mrtvého kódu vyžaduje cross-file analýzu. Funkce, která se zdá nepoužívaná ve svém vlastním souboru, může být jediným exportem utilitního modulu, na kterém závisí 30 dalších souborů. Naopak funkce, která je importována v jiném souboru, může být importována pouze pro účely typové kontroly a nikdy skutečně volána za běhu.</p>

<p>AigisCode k tomu přistupuje budováním kompletního grafu závislostí z AST analýzy tree-sitter. Každý import, každé volání funkce, každá instanciace třídy je sledována napříč celým codebase. Detektor mrtvého kódu poté identifikuje symboly, které nemají žádné příchozí reference v grafu, a označí je úrovněmi spolehlivosti podle toho, jak si je analýza jistá.</p>

<p>Pro frameworky s konvenčně založeným objevováním kódu — jako jsou Django view funkce odkazované v URL konfiguracích nebo Laravel service providery registrované v konfiguračních souborech — policy systém AigisCode umožňuje týmům tyto vstupní body explicitně označit, čímž se zabrání false positives bez úplného vypnutí detektoru.</p>

<h2 id="practical-cleanup-workflow">Praktický postup čištění</h2>

<p>Čištění mrtvého kódu by mělo být systematické, ne heroické. Zde je postup, který mohou týmy následovat.</p>

<p><strong>Krok 1: Základní sken.</strong> Spusťte <code>aigiscode analyze /path/to/project</code> pro vygenerování úvodního reportu. Prohlédněte sekci <code>dead_code</code> ve výstupu JSON.</p>

<p><strong>Krok 2: Třídění podle spolehlivosti.</strong> Začněte s nálezy s vysokou spolehlivostí. To jsou symboly, u kterých je deterministická analýza jistá, že jsou nepoužívané. Osiřelé soubory a nepoužívané importy bez vzorců dynamického odkazování mají typicky vysokou spolehlivost.</p>

<p><strong>Krok 3: Vzorkování a ověření.</strong> Před hromadným mazáním ručně ověřte vzorek nálezů. Zkontrolujte, že kód je skutečně nedosažitelný. Hledejte dynamické reference, konfigurací řízené objevování a vzorce reflexe, které mohla statická analýza přehlédnout.</p>

<p><strong>Krok 4: Mazání v malých dávkách.</strong> Odstraňujte mrtvý kód ve cílených pull requestech, jednu kategorii nebo modul najednou. To činí code review zvladatelným a umožňuje snadné vrácení, pokud se nález ukáže jako false positive.</p>

<p><strong>Krok 5: Aktualizace policy.</strong> Pro false positives přidejte pravidla výjimek do <code>.aigiscode/rules.json</code>, aby se v budoucích skenech znovu neobjevovaly. Pro vzorce false positives je zakódujte do <code>.aigiscode/policy.json</code>, aby se detektor naučil podobné vzory přeskakovat.</p>

<p><strong>Krok 6: Prevence regrese.</strong> Přidejte AigisCode do vaší CI pipeline. Označujte nové zavedení mrtvého kódu v pull requestech. Cílem není nulový mrtvý kód, ale konzistentní trend směrem k méně mrtvého kódu.</p>

<h2 id="the-bottom-line">Závěr</h2>

<p>Mrtvý kód není nouzový stav. Nezpůsobuje produkční výpadky ani ztrátu dat. Ale je to pomalá, narůstající daň na každou inženýrskou činnost. Zpomaluje onboarding, nafukuje bundly, mate vývojáře, rozšiřuje bezpečnostní plochu a ztěžuje řešení všech ostatních forem technického dluhu. Dobrou zprávou je, že jeho detekce je vyřešený problém. Nástroje existují. Postup je přímočarý. Jedinou překážkou je učinit rozhodnutí začít.</p>
`,
      fr: `
<p>Dead code is the sediment of software development. Every feature that gets replaced, every experiment that does not ship, every refactoring that moves logic to a new location but forgets to remove the old one, they all leave behind code that is no longer executed. And unlike a broken test or a compile error, dead code does not announce its presence. It just sits there, quietly increasing the size, complexity, and maintenance burden of your codebase.</p>

<h2 id="how-dead-code-accumulates">How Dead Code Accumulates</h2>

<p>Dead code rarely appears through deliberate action. No developer writes a function with the intent of never calling it. Instead, dead code emerges through the natural lifecycle of software development.</p>

<p><strong>Feature replacement.</strong> A team builds version 2 of a feature, carefully routes all traffic to the new implementation, but never deletes the version 1 code because "we might need to roll back." Six months later, the rollback window has passed, but the old code remains because nobody remembers which files are safe to delete.</p>

<p><strong>Refactoring residue.</strong> A developer moves a utility function from <code>helpers.py</code> to <code>utils/string_helpers.py</code>. All call sites are updated. But the original function definition in <code>helpers.py</code> is left in place because the developer was not certain nothing else referenced it. The IDE shows no direct callers, but what about dynamic imports? What about tests? The safer choice feels like leaving it alone.</p>

<p><strong>Copy-paste evolution.</strong> A method is copied from one class to another to make a quick fix. The original is never removed. Over time, both copies evolve independently, and developers are unsure which one is the "real" version.</p>

<p><strong>Configuration drift.</strong> Environment variables, feature flags, and config keys accumulate over multiple release cycles. Old flags for features that shipped months ago remain in the config files, and the code that checks them remains in the application, even though the flag is always set to the same value.</p>

<h2 id="the-real-costs">The Real Costs of Dead Code</h2>

<h3>Developer Confusion</h3>

<p>The most immediate cost of dead code is cognitive load. A new developer joins the team, opens the codebase, and encounters a module with 15 classes. Five of them are dead. But the new developer does not know that. They spend time reading, understanding, and forming mental models of code that does not matter. Worse, they might build new features on top of dead code, creating dependencies on zombie implementations that nobody maintains.</p>

<p>This is not a theoretical concern. A 2025 survey of 1,200 developers by JetBrains found that <strong>63% of respondents</strong> cited "understanding unfamiliar code" as their biggest productivity bottleneck. Dead code directly inflates the amount of unfamiliar code that developers must navigate.</p>

<h3>Larger Bundles and Slower Builds</h3>

<p>In frontend applications, dead code directly impacts user experience. Unused components, abandoned utility functions, and orphan modules all get bundled into production JavaScript if tree-shaking fails to eliminate them. And tree-shaking has limits. If a module has side effects, or if the dead code is referenced through dynamic imports that the bundler cannot statically analyze, it ends up in the bundle.</p>

<p>On the backend, dead code increases build times, test execution time, and container image sizes. Every dead file is another file that the test runner must discover and skip. Every dead import is another module that must be resolved during startup.</p>

<h3>Expanded Security Surface</h3>

<p>Dead code that is still present in the repository is still present in production. If that dead code contains a vulnerability, such as an old API client that uses an insecure authentication method, or a deprecated utility that performs unsanitized string interpolation, the vulnerability is exploitable even if no active code path reaches it. An attacker who gains the ability to call arbitrary functions does not care whether those functions are "dead" from the application's perspective.</p>

<h3>CI and Tooling Overhead</h3>

<p>Every dead file adds to the workload of your continuous integration pipeline. Linters analyze it. Type checkers process it. Code coverage tools report on it, showing artificially low coverage percentages because dead code has zero test coverage by definition. Security scanners flag vulnerabilities in it. And developers spend time triaging these findings, only to discover they are about code that nobody uses.</p>

<h2 id="types-of-dead-code">Types of Dead Code</h2>

<p>Dead code is not monolithic. AigisCode detects several distinct categories, each with different detection strategies and risk profiles.</p>

<p><strong>Unused imports</strong> are the most common and easiest to detect. A module imports a symbol that is never used in the rest of the file. Most linters catch these at the single-file level, but cross-module unused imports, where a module re-exports a symbol that nobody else imports, require codebase-wide analysis.</p>

<p><strong>Unreferenced methods</strong> are class methods that are defined but never called from anywhere in the codebase. These are harder to detect because methods might be called dynamically through reflection, decorators, or framework conventions. AigisCode uses confidence levels to distinguish between methods that are definitively unused and those that might be called through dynamic dispatch.</p>

<p><strong>Abandoned classes</strong> are entire classes that are never instantiated or referenced. These often result from feature replacement where the new implementation uses a different class name or architecture.</p>

<p><strong>Orphan files</strong> are files with no inbound dependencies. No other file imports from them, no configuration references them, and no test targets them. These are often the clearest indicators of dead code because a file with no inbound dependencies can be safely deleted without affecting any other part of the codebase.</p>

<p><strong>Orphan properties</strong> are class attributes or object properties that are assigned but never read. These accumulate as data models evolve and fields are added for features that are later abandoned.</p>

<h2 id="detection-strategies">Detection Strategies</h2>

<p>Detecting dead code reliably requires cross-file analysis. A function that appears unused in its own file might be the only export of a utility module that 30 other files depend on. Conversely, a function that is imported in another file might only be imported for type-checking purposes and never actually called at runtime.</p>

<p>AigisCode approaches this by building a complete dependency graph from tree-sitter AST analysis. Every import, every function call, every class instantiation is tracked across the entire codebase. The dead code detector then identifies symbols that have no inbound references in the graph, flagging them with confidence levels based on how certain the analysis is.</p>

<p>For frameworks with convention-based code discovery, like Django's view functions referenced in URL configs or Laravel's service providers registered in config files, AigisCode's policy system allows teams to mark these entry points explicitly, preventing false positives without disabling the detector entirely.</p>

<h2 id="practical-cleanup-workflow">A Practical Cleanup Workflow</h2>

<p>Cleaning up dead code should be systematic, not heroic. Here is a workflow that teams can follow.</p>

<p><strong>Step 1: Baseline scan.</strong> Run <code>aigiscode analyze /path/to/project</code> to generate the initial report. Review the <code>dead_code</code> section of the JSON output.</p>

<p><strong>Step 2: Triage by confidence.</strong> Start with high-confidence findings. These are symbols that the deterministic analysis is certain are unused. Orphan files and unused imports with no dynamic reference patterns are typically high confidence.</p>

<p><strong>Step 3: Sample and verify.</strong> Before bulk-deleting, manually verify a sample of findings. Check that the code is truly unreachable. Search for dynamic references, configuration-based discovery, and reflection patterns that the static analysis might have missed.</p>

<p><strong>Step 4: Delete in small batches.</strong> Remove dead code in focused pull requests, one category or module at a time. This makes code review manageable and allows for easy reversion if a finding turns out to be a false positive.</p>

<p><strong>Step 5: Update policy.</strong> For false positives, add exclusion rules to <code>.aigiscode/rules.json</code> so they do not reappear in future scans. For patterns of false positives, encode them in <code>.aigiscode/policy.json</code> so the detector learns to skip similar patterns.</p>

<p><strong>Step 6: Prevent regression.</strong> Add AigisCode to your CI pipeline. Flag new dead code introductions in pull requests. The goal is not zero dead code but a consistent trend toward less of it.</p>

<h2 id="the-bottom-line">The Bottom Line</h2>

<p>Dead code is not an emergency. It does not cause production outages or data loss. But it is a slow, compounding tax on every engineering activity. It slows onboarding, inflates bundles, confuses developers, expands the security surface, and makes every other form of technical debt harder to address. The good news is that detecting it is a solved problem. The tools exist. The workflow is straightforward. The only barrier is making the decision to start.</p>
`,
      es: `
<p>Dead code is the sediment of software development. Every feature that gets replaced, every experiment that does not ship, every refactoring that moves logic to a new location but forgets to remove the old one, they all leave behind code that is no longer executed. And unlike a broken test or a compile error, dead code does not announce its presence. It just sits there, quietly increasing the size, complexity, and maintenance burden of your codebase.</p>

<h2 id="how-dead-code-accumulates">How Dead Code Accumulates</h2>

<p>Dead code rarely appears through deliberate action. No developer writes a function with the intent of never calling it. Instead, dead code emerges through the natural lifecycle of software development.</p>

<p><strong>Feature replacement.</strong> A team builds version 2 of a feature, carefully routes all traffic to the new implementation, but never deletes the version 1 code because "we might need to roll back." Six months later, the rollback window has passed, but the old code remains because nobody remembers which files are safe to delete.</p>

<p><strong>Refactoring residue.</strong> A developer moves a utility function from <code>helpers.py</code> to <code>utils/string_helpers.py</code>. All call sites are updated. But the original function definition in <code>helpers.py</code> is left in place because the developer was not certain nothing else referenced it. The IDE shows no direct callers, but what about dynamic imports? What about tests? The safer choice feels like leaving it alone.</p>

<p><strong>Copy-paste evolution.</strong> A method is copied from one class to another to make a quick fix. The original is never removed. Over time, both copies evolve independently, and developers are unsure which one is the "real" version.</p>

<p><strong>Configuration drift.</strong> Environment variables, feature flags, and config keys accumulate over multiple release cycles. Old flags for features that shipped months ago remain in the config files, and the code that checks them remains in the application, even though the flag is always set to the same value.</p>

<h2 id="the-real-costs">The Real Costs of Dead Code</h2>

<h3>Developer Confusion</h3>

<p>The most immediate cost of dead code is cognitive load. A new developer joins the team, opens the codebase, and encounters a module with 15 classes. Five of them are dead. But the new developer does not know that. They spend time reading, understanding, and forming mental models of code that does not matter. Worse, they might build new features on top of dead code, creating dependencies on zombie implementations that nobody maintains.</p>

<p>This is not a theoretical concern. A 2025 survey of 1,200 developers by JetBrains found that <strong>63% of respondents</strong> cited "understanding unfamiliar code" as their biggest productivity bottleneck. Dead code directly inflates the amount of unfamiliar code that developers must navigate.</p>

<h3>Larger Bundles and Slower Builds</h3>

<p>In frontend applications, dead code directly impacts user experience. Unused components, abandoned utility functions, and orphan modules all get bundled into production JavaScript if tree-shaking fails to eliminate them. And tree-shaking has limits. If a module has side effects, or if the dead code is referenced through dynamic imports that the bundler cannot statically analyze, it ends up in the bundle.</p>

<p>On the backend, dead code increases build times, test execution time, and container image sizes. Every dead file is another file that the test runner must discover and skip. Every dead import is another module that must be resolved during startup.</p>

<h3>Expanded Security Surface</h3>

<p>Dead code that is still present in the repository is still present in production. If that dead code contains a vulnerability, such as an old API client that uses an insecure authentication method, or a deprecated utility that performs unsanitized string interpolation, the vulnerability is exploitable even if no active code path reaches it. An attacker who gains the ability to call arbitrary functions does not care whether those functions are "dead" from the application's perspective.</p>

<h3>CI and Tooling Overhead</h3>

<p>Every dead file adds to the workload of your continuous integration pipeline. Linters analyze it. Type checkers process it. Code coverage tools report on it, showing artificially low coverage percentages because dead code has zero test coverage by definition. Security scanners flag vulnerabilities in it. And developers spend time triaging these findings, only to discover they are about code that nobody uses.</p>

<h2 id="types-of-dead-code">Types of Dead Code</h2>

<p>Dead code is not monolithic. AigisCode detects several distinct categories, each with different detection strategies and risk profiles.</p>

<p><strong>Unused imports</strong> are the most common and easiest to detect. A module imports a symbol that is never used in the rest of the file. Most linters catch these at the single-file level, but cross-module unused imports, where a module re-exports a symbol that nobody else imports, require codebase-wide analysis.</p>

<p><strong>Unreferenced methods</strong> are class methods that are defined but never called from anywhere in the codebase. These are harder to detect because methods might be called dynamically through reflection, decorators, or framework conventions. AigisCode uses confidence levels to distinguish between methods that are definitively unused and those that might be called through dynamic dispatch.</p>

<p><strong>Abandoned classes</strong> are entire classes that are never instantiated or referenced. These often result from feature replacement where the new implementation uses a different class name or architecture.</p>

<p><strong>Orphan files</strong> are files with no inbound dependencies. No other file imports from them, no configuration references them, and no test targets them. These are often the clearest indicators of dead code because a file with no inbound dependencies can be safely deleted without affecting any other part of the codebase.</p>

<p><strong>Orphan properties</strong> are class attributes or object properties that are assigned but never read. These accumulate as data models evolve and fields are added for features that are later abandoned.</p>

<h2 id="detection-strategies">Detection Strategies</h2>

<p>Detecting dead code reliably requires cross-file analysis. A function that appears unused in its own file might be the only export of a utility module that 30 other files depend on. Conversely, a function that is imported in another file might only be imported for type-checking purposes and never actually called at runtime.</p>

<p>AigisCode approaches this by building a complete dependency graph from tree-sitter AST analysis. Every import, every function call, every class instantiation is tracked across the entire codebase. The dead code detector then identifies symbols that have no inbound references in the graph, flagging them with confidence levels based on how certain the analysis is.</p>

<p>For frameworks with convention-based code discovery, like Django's view functions referenced in URL configs or Laravel's service providers registered in config files, AigisCode's policy system allows teams to mark these entry points explicitly, preventing false positives without disabling the detector entirely.</p>

<h2 id="practical-cleanup-workflow">A Practical Cleanup Workflow</h2>

<p>Cleaning up dead code should be systematic, not heroic. Here is a workflow that teams can follow.</p>

<p><strong>Step 1: Baseline scan.</strong> Run <code>aigiscode analyze /path/to/project</code> to generate the initial report. Review the <code>dead_code</code> section of the JSON output.</p>

<p><strong>Step 2: Triage by confidence.</strong> Start with high-confidence findings. These are symbols that the deterministic analysis is certain are unused. Orphan files and unused imports with no dynamic reference patterns are typically high confidence.</p>

<p><strong>Step 3: Sample and verify.</strong> Before bulk-deleting, manually verify a sample of findings. Check that the code is truly unreachable. Search for dynamic references, configuration-based discovery, and reflection patterns that the static analysis might have missed.</p>

<p><strong>Step 4: Delete in small batches.</strong> Remove dead code in focused pull requests, one category or module at a time. This makes code review manageable and allows for easy reversion if a finding turns out to be a false positive.</p>

<p><strong>Step 5: Update policy.</strong> For false positives, add exclusion rules to <code>.aigiscode/rules.json</code> so they do not reappear in future scans. For patterns of false positives, encode them in <code>.aigiscode/policy.json</code> so the detector learns to skip similar patterns.</p>

<p><strong>Step 6: Prevent regression.</strong> Add AigisCode to your CI pipeline. Flag new dead code introductions in pull requests. The goal is not zero dead code but a consistent trend toward less of it.</p>

<h2 id="the-bottom-line">The Bottom Line</h2>

<p>Dead code is not an emergency. It does not cause production outages or data loss. But it is a slow, compounding tax on every engineering activity. It slows onboarding, inflates bundles, confuses developers, expands the security surface, and makes every other form of technical debt harder to address. The good news is that detecting it is a solved problem. The tools exist. The workflow is straightforward. The only barrier is making the decision to start.</p>
`,
      zh: `
<p>Dead code is the sediment of software development. Every feature that gets replaced, every experiment that does not ship, every refactoring that moves logic to a new location but forgets to remove the old one, they all leave behind code that is no longer executed. And unlike a broken test or a compile error, dead code does not announce its presence. It just sits there, quietly increasing the size, complexity, and maintenance burden of your codebase.</p>

<h2 id="how-dead-code-accumulates">How Dead Code Accumulates</h2>

<p>Dead code rarely appears through deliberate action. No developer writes a function with the intent of never calling it. Instead, dead code emerges through the natural lifecycle of software development.</p>

<p><strong>Feature replacement.</strong> A team builds version 2 of a feature, carefully routes all traffic to the new implementation, but never deletes the version 1 code because "we might need to roll back." Six months later, the rollback window has passed, but the old code remains because nobody remembers which files are safe to delete.</p>

<p><strong>Refactoring residue.</strong> A developer moves a utility function from <code>helpers.py</code> to <code>utils/string_helpers.py</code>. All call sites are updated. But the original function definition in <code>helpers.py</code> is left in place because the developer was not certain nothing else referenced it. The IDE shows no direct callers, but what about dynamic imports? What about tests? The safer choice feels like leaving it alone.</p>

<p><strong>Copy-paste evolution.</strong> A method is copied from one class to another to make a quick fix. The original is never removed. Over time, both copies evolve independently, and developers are unsure which one is the "real" version.</p>

<p><strong>Configuration drift.</strong> Environment variables, feature flags, and config keys accumulate over multiple release cycles. Old flags for features that shipped months ago remain in the config files, and the code that checks them remains in the application, even though the flag is always set to the same value.</p>

<h2 id="the-real-costs">The Real Costs of Dead Code</h2>

<h3>Developer Confusion</h3>

<p>The most immediate cost of dead code is cognitive load. A new developer joins the team, opens the codebase, and encounters a module with 15 classes. Five of them are dead. But the new developer does not know that. They spend time reading, understanding, and forming mental models of code that does not matter. Worse, they might build new features on top of dead code, creating dependencies on zombie implementations that nobody maintains.</p>

<p>This is not a theoretical concern. A 2025 survey of 1,200 developers by JetBrains found that <strong>63% of respondents</strong> cited "understanding unfamiliar code" as their biggest productivity bottleneck. Dead code directly inflates the amount of unfamiliar code that developers must navigate.</p>

<h3>Larger Bundles and Slower Builds</h3>

<p>In frontend applications, dead code directly impacts user experience. Unused components, abandoned utility functions, and orphan modules all get bundled into production JavaScript if tree-shaking fails to eliminate them. And tree-shaking has limits. If a module has side effects, or if the dead code is referenced through dynamic imports that the bundler cannot statically analyze, it ends up in the bundle.</p>

<p>On the backend, dead code increases build times, test execution time, and container image sizes. Every dead file is another file that the test runner must discover and skip. Every dead import is another module that must be resolved during startup.</p>

<h3>Expanded Security Surface</h3>

<p>Dead code that is still present in the repository is still present in production. If that dead code contains a vulnerability, such as an old API client that uses an insecure authentication method, or a deprecated utility that performs unsanitized string interpolation, the vulnerability is exploitable even if no active code path reaches it. An attacker who gains the ability to call arbitrary functions does not care whether those functions are "dead" from the application's perspective.</p>

<h3>CI and Tooling Overhead</h3>

<p>Every dead file adds to the workload of your continuous integration pipeline. Linters analyze it. Type checkers process it. Code coverage tools report on it, showing artificially low coverage percentages because dead code has zero test coverage by definition. Security scanners flag vulnerabilities in it. And developers spend time triaging these findings, only to discover they are about code that nobody uses.</p>

<h2 id="types-of-dead-code">Types of Dead Code</h2>

<p>Dead code is not monolithic. AigisCode detects several distinct categories, each with different detection strategies and risk profiles.</p>

<p><strong>Unused imports</strong> are the most common and easiest to detect. A module imports a symbol that is never used in the rest of the file. Most linters catch these at the single-file level, but cross-module unused imports, where a module re-exports a symbol that nobody else imports, require codebase-wide analysis.</p>

<p><strong>Unreferenced methods</strong> are class methods that are defined but never called from anywhere in the codebase. These are harder to detect because methods might be called dynamically through reflection, decorators, or framework conventions. AigisCode uses confidence levels to distinguish between methods that are definitively unused and those that might be called through dynamic dispatch.</p>

<p><strong>Abandoned classes</strong> are entire classes that are never instantiated or referenced. These often result from feature replacement where the new implementation uses a different class name or architecture.</p>

<p><strong>Orphan files</strong> are files with no inbound dependencies. No other file imports from them, no configuration references them, and no test targets them. These are often the clearest indicators of dead code because a file with no inbound dependencies can be safely deleted without affecting any other part of the codebase.</p>

<p><strong>Orphan properties</strong> are class attributes or object properties that are assigned but never read. These accumulate as data models evolve and fields are added for features that are later abandoned.</p>

<h2 id="detection-strategies">Detection Strategies</h2>

<p>Detecting dead code reliably requires cross-file analysis. A function that appears unused in its own file might be the only export of a utility module that 30 other files depend on. Conversely, a function that is imported in another file might only be imported for type-checking purposes and never actually called at runtime.</p>

<p>AigisCode approaches this by building a complete dependency graph from tree-sitter AST analysis. Every import, every function call, every class instantiation is tracked across the entire codebase. The dead code detector then identifies symbols that have no inbound references in the graph, flagging them with confidence levels based on how certain the analysis is.</p>

<p>For frameworks with convention-based code discovery, like Django's view functions referenced in URL configs or Laravel's service providers registered in config files, AigisCode's policy system allows teams to mark these entry points explicitly, preventing false positives without disabling the detector entirely.</p>

<h2 id="practical-cleanup-workflow">A Practical Cleanup Workflow</h2>

<p>Cleaning up dead code should be systematic, not heroic. Here is a workflow that teams can follow.</p>

<p><strong>Step 1: Baseline scan.</strong> Run <code>aigiscode analyze /path/to/project</code> to generate the initial report. Review the <code>dead_code</code> section of the JSON output.</p>

<p><strong>Step 2: Triage by confidence.</strong> Start with high-confidence findings. These are symbols that the deterministic analysis is certain are unused. Orphan files and unused imports with no dynamic reference patterns are typically high confidence.</p>

<p><strong>Step 3: Sample and verify.</strong> Before bulk-deleting, manually verify a sample of findings. Check that the code is truly unreachable. Search for dynamic references, configuration-based discovery, and reflection patterns that the static analysis might have missed.</p>

<p><strong>Step 4: Delete in small batches.</strong> Remove dead code in focused pull requests, one category or module at a time. This makes code review manageable and allows for easy reversion if a finding turns out to be a false positive.</p>

<p><strong>Step 5: Update policy.</strong> For false positives, add exclusion rules to <code>.aigiscode/rules.json</code> so they do not reappear in future scans. For patterns of false positives, encode them in <code>.aigiscode/policy.json</code> so the detector learns to skip similar patterns.</p>

<p><strong>Step 6: Prevent regression.</strong> Add AigisCode to your CI pipeline. Flag new dead code introductions in pull requests. The goal is not zero dead code but a consistent trend toward less of it.</p>

<h2 id="the-bottom-line">The Bottom Line</h2>

<p>Dead code is not an emergency. It does not cause production outages or data loss. But it is a slow, compounding tax on every engineering activity. It slows onboarding, inflates bundles, confuses developers, expands the security surface, and makes every other form of technical debt harder to address. The good news is that detecting it is a solved problem. The tools exist. The workflow is straightforward. The only barrier is making the decision to start.</p>
`,
      hi: `
<p>Dead code is the sediment of software development. Every feature that gets replaced, every experiment that does not ship, every refactoring that moves logic to a new location but forgets to remove the old one, they all leave behind code that is no longer executed. And unlike a broken test or a compile error, dead code does not announce its presence. It just sits there, quietly increasing the size, complexity, and maintenance burden of your codebase.</p>

<h2 id="how-dead-code-accumulates">How Dead Code Accumulates</h2>

<p>Dead code rarely appears through deliberate action. No developer writes a function with the intent of never calling it. Instead, dead code emerges through the natural lifecycle of software development.</p>

<p><strong>Feature replacement.</strong> A team builds version 2 of a feature, carefully routes all traffic to the new implementation, but never deletes the version 1 code because "we might need to roll back." Six months later, the rollback window has passed, but the old code remains because nobody remembers which files are safe to delete.</p>

<p><strong>Refactoring residue.</strong> A developer moves a utility function from <code>helpers.py</code> to <code>utils/string_helpers.py</code>. All call sites are updated. But the original function definition in <code>helpers.py</code> is left in place because the developer was not certain nothing else referenced it. The IDE shows no direct callers, but what about dynamic imports? What about tests? The safer choice feels like leaving it alone.</p>

<p><strong>Copy-paste evolution.</strong> A method is copied from one class to another to make a quick fix. The original is never removed. Over time, both copies evolve independently, and developers are unsure which one is the "real" version.</p>

<p><strong>Configuration drift.</strong> Environment variables, feature flags, and config keys accumulate over multiple release cycles. Old flags for features that shipped months ago remain in the config files, and the code that checks them remains in the application, even though the flag is always set to the same value.</p>

<h2 id="the-real-costs">The Real Costs of Dead Code</h2>

<h3>Developer Confusion</h3>

<p>The most immediate cost of dead code is cognitive load. A new developer joins the team, opens the codebase, and encounters a module with 15 classes. Five of them are dead. But the new developer does not know that. They spend time reading, understanding, and forming mental models of code that does not matter. Worse, they might build new features on top of dead code, creating dependencies on zombie implementations that nobody maintains.</p>

<p>This is not a theoretical concern. A 2025 survey of 1,200 developers by JetBrains found that <strong>63% of respondents</strong> cited "understanding unfamiliar code" as their biggest productivity bottleneck. Dead code directly inflates the amount of unfamiliar code that developers must navigate.</p>

<h3>Larger Bundles and Slower Builds</h3>

<p>In frontend applications, dead code directly impacts user experience. Unused components, abandoned utility functions, and orphan modules all get bundled into production JavaScript if tree-shaking fails to eliminate them. And tree-shaking has limits. If a module has side effects, or if the dead code is referenced through dynamic imports that the bundler cannot statically analyze, it ends up in the bundle.</p>

<p>On the backend, dead code increases build times, test execution time, and container image sizes. Every dead file is another file that the test runner must discover and skip. Every dead import is another module that must be resolved during startup.</p>

<h3>Expanded Security Surface</h3>

<p>Dead code that is still present in the repository is still present in production. If that dead code contains a vulnerability, such as an old API client that uses an insecure authentication method, or a deprecated utility that performs unsanitized string interpolation, the vulnerability is exploitable even if no active code path reaches it. An attacker who gains the ability to call arbitrary functions does not care whether those functions are "dead" from the application's perspective.</p>

<h3>CI and Tooling Overhead</h3>

<p>Every dead file adds to the workload of your continuous integration pipeline. Linters analyze it. Type checkers process it. Code coverage tools report on it, showing artificially low coverage percentages because dead code has zero test coverage by definition. Security scanners flag vulnerabilities in it. And developers spend time triaging these findings, only to discover they are about code that nobody uses.</p>

<h2 id="types-of-dead-code">Types of Dead Code</h2>

<p>Dead code is not monolithic. AigisCode detects several distinct categories, each with different detection strategies and risk profiles.</p>

<p><strong>Unused imports</strong> are the most common and easiest to detect. A module imports a symbol that is never used in the rest of the file. Most linters catch these at the single-file level, but cross-module unused imports, where a module re-exports a symbol that nobody else imports, require codebase-wide analysis.</p>

<p><strong>Unreferenced methods</strong> are class methods that are defined but never called from anywhere in the codebase. These are harder to detect because methods might be called dynamically through reflection, decorators, or framework conventions. AigisCode uses confidence levels to distinguish between methods that are definitively unused and those that might be called through dynamic dispatch.</p>

<p><strong>Abandoned classes</strong> are entire classes that are never instantiated or referenced. These often result from feature replacement where the new implementation uses a different class name or architecture.</p>

<p><strong>Orphan files</strong> are files with no inbound dependencies. No other file imports from them, no configuration references them, and no test targets them. These are often the clearest indicators of dead code because a file with no inbound dependencies can be safely deleted without affecting any other part of the codebase.</p>

<p><strong>Orphan properties</strong> are class attributes or object properties that are assigned but never read. These accumulate as data models evolve and fields are added for features that are later abandoned.</p>

<h2 id="detection-strategies">Detection Strategies</h2>

<p>Detecting dead code reliably requires cross-file analysis. A function that appears unused in its own file might be the only export of a utility module that 30 other files depend on. Conversely, a function that is imported in another file might only be imported for type-checking purposes and never actually called at runtime.</p>

<p>AigisCode approaches this by building a complete dependency graph from tree-sitter AST analysis. Every import, every function call, every class instantiation is tracked across the entire codebase. The dead code detector then identifies symbols that have no inbound references in the graph, flagging them with confidence levels based on how certain the analysis is.</p>

<p>For frameworks with convention-based code discovery, like Django's view functions referenced in URL configs or Laravel's service providers registered in config files, AigisCode's policy system allows teams to mark these entry points explicitly, preventing false positives without disabling the detector entirely.</p>

<h2 id="practical-cleanup-workflow">A Practical Cleanup Workflow</h2>

<p>Cleaning up dead code should be systematic, not heroic. Here is a workflow that teams can follow.</p>

<p><strong>Step 1: Baseline scan.</strong> Run <code>aigiscode analyze /path/to/project</code> to generate the initial report. Review the <code>dead_code</code> section of the JSON output.</p>

<p><strong>Step 2: Triage by confidence.</strong> Start with high-confidence findings. These are symbols that the deterministic analysis is certain are unused. Orphan files and unused imports with no dynamic reference patterns are typically high confidence.</p>

<p><strong>Step 3: Sample and verify.</strong> Before bulk-deleting, manually verify a sample of findings. Check that the code is truly unreachable. Search for dynamic references, configuration-based discovery, and reflection patterns that the static analysis might have missed.</p>

<p><strong>Step 4: Delete in small batches.</strong> Remove dead code in focused pull requests, one category or module at a time. This makes code review manageable and allows for easy reversion if a finding turns out to be a false positive.</p>

<p><strong>Step 5: Update policy.</strong> For false positives, add exclusion rules to <code>.aigiscode/rules.json</code> so they do not reappear in future scans. For patterns of false positives, encode them in <code>.aigiscode/policy.json</code> so the detector learns to skip similar patterns.</p>

<p><strong>Step 6: Prevent regression.</strong> Add AigisCode to your CI pipeline. Flag new dead code introductions in pull requests. The goal is not zero dead code but a consistent trend toward less of it.</p>

<h2 id="the-bottom-line">The Bottom Line</h2>

<p>Dead code is not an emergency. It does not cause production outages or data loss. But it is a slow, compounding tax on every engineering activity. It slows onboarding, inflates bundles, confuses developers, expands the security surface, and makes every other form of technical debt harder to address. The good news is that detecting it is a solved problem. The tools exist. The workflow is straightforward. The only barrier is making the decision to start.</p>
`,
      pt: `
<p>Dead code is the sediment of software development. Every feature that gets replaced, every experiment that does not ship, every refactoring that moves logic to a new location but forgets to remove the old one, they all leave behind code that is no longer executed. And unlike a broken test or a compile error, dead code does not announce its presence. It just sits there, quietly increasing the size, complexity, and maintenance burden of your codebase.</p>

<h2 id="how-dead-code-accumulates">How Dead Code Accumulates</h2>

<p>Dead code rarely appears through deliberate action. No developer writes a function with the intent of never calling it. Instead, dead code emerges through the natural lifecycle of software development.</p>

<p><strong>Feature replacement.</strong> A team builds version 2 of a feature, carefully routes all traffic to the new implementation, but never deletes the version 1 code because "we might need to roll back." Six months later, the rollback window has passed, but the old code remains because nobody remembers which files are safe to delete.</p>

<p><strong>Refactoring residue.</strong> A developer moves a utility function from <code>helpers.py</code> to <code>utils/string_helpers.py</code>. All call sites are updated. But the original function definition in <code>helpers.py</code> is left in place because the developer was not certain nothing else referenced it. The IDE shows no direct callers, but what about dynamic imports? What about tests? The safer choice feels like leaving it alone.</p>

<p><strong>Copy-paste evolution.</strong> A method is copied from one class to another to make a quick fix. The original is never removed. Over time, both copies evolve independently, and developers are unsure which one is the "real" version.</p>

<p><strong>Configuration drift.</strong> Environment variables, feature flags, and config keys accumulate over multiple release cycles. Old flags for features that shipped months ago remain in the config files, and the code that checks them remains in the application, even though the flag is always set to the same value.</p>

<h2 id="the-real-costs">The Real Costs of Dead Code</h2>

<h3>Developer Confusion</h3>

<p>The most immediate cost of dead code is cognitive load. A new developer joins the team, opens the codebase, and encounters a module with 15 classes. Five of them are dead. But the new developer does not know that. They spend time reading, understanding, and forming mental models of code that does not matter. Worse, they might build new features on top of dead code, creating dependencies on zombie implementations that nobody maintains.</p>

<p>This is not a theoretical concern. A 2025 survey of 1,200 developers by JetBrains found that <strong>63% of respondents</strong> cited "understanding unfamiliar code" as their biggest productivity bottleneck. Dead code directly inflates the amount of unfamiliar code that developers must navigate.</p>

<h3>Larger Bundles and Slower Builds</h3>

<p>In frontend applications, dead code directly impacts user experience. Unused components, abandoned utility functions, and orphan modules all get bundled into production JavaScript if tree-shaking fails to eliminate them. And tree-shaking has limits. If a module has side effects, or if the dead code is referenced through dynamic imports that the bundler cannot statically analyze, it ends up in the bundle.</p>

<p>On the backend, dead code increases build times, test execution time, and container image sizes. Every dead file is another file that the test runner must discover and skip. Every dead import is another module that must be resolved during startup.</p>

<h3>Expanded Security Surface</h3>

<p>Dead code that is still present in the repository is still present in production. If that dead code contains a vulnerability, such as an old API client that uses an insecure authentication method, or a deprecated utility that performs unsanitized string interpolation, the vulnerability is exploitable even if no active code path reaches it. An attacker who gains the ability to call arbitrary functions does not care whether those functions are "dead" from the application's perspective.</p>

<h3>CI and Tooling Overhead</h3>

<p>Every dead file adds to the workload of your continuous integration pipeline. Linters analyze it. Type checkers process it. Code coverage tools report on it, showing artificially low coverage percentages because dead code has zero test coverage by definition. Security scanners flag vulnerabilities in it. And developers spend time triaging these findings, only to discover they are about code that nobody uses.</p>

<h2 id="types-of-dead-code">Types of Dead Code</h2>

<p>Dead code is not monolithic. AigisCode detects several distinct categories, each with different detection strategies and risk profiles.</p>

<p><strong>Unused imports</strong> are the most common and easiest to detect. A module imports a symbol that is never used in the rest of the file. Most linters catch these at the single-file level, but cross-module unused imports, where a module re-exports a symbol that nobody else imports, require codebase-wide analysis.</p>

<p><strong>Unreferenced methods</strong> are class methods that are defined but never called from anywhere in the codebase. These are harder to detect because methods might be called dynamically through reflection, decorators, or framework conventions. AigisCode uses confidence levels to distinguish between methods that are definitively unused and those that might be called through dynamic dispatch.</p>

<p><strong>Abandoned classes</strong> are entire classes that are never instantiated or referenced. These often result from feature replacement where the new implementation uses a different class name or architecture.</p>

<p><strong>Orphan files</strong> are files with no inbound dependencies. No other file imports from them, no configuration references them, and no test targets them. These are often the clearest indicators of dead code because a file with no inbound dependencies can be safely deleted without affecting any other part of the codebase.</p>

<p><strong>Orphan properties</strong> are class attributes or object properties that are assigned but never read. These accumulate as data models evolve and fields are added for features that are later abandoned.</p>

<h2 id="detection-strategies">Detection Strategies</h2>

<p>Detecting dead code reliably requires cross-file analysis. A function that appears unused in its own file might be the only export of a utility module that 30 other files depend on. Conversely, a function that is imported in another file might only be imported for type-checking purposes and never actually called at runtime.</p>

<p>AigisCode approaches this by building a complete dependency graph from tree-sitter AST analysis. Every import, every function call, every class instantiation is tracked across the entire codebase. The dead code detector then identifies symbols that have no inbound references in the graph, flagging them with confidence levels based on how certain the analysis is.</p>

<p>For frameworks with convention-based code discovery, like Django's view functions referenced in URL configs or Laravel's service providers registered in config files, AigisCode's policy system allows teams to mark these entry points explicitly, preventing false positives without disabling the detector entirely.</p>

<h2 id="practical-cleanup-workflow">A Practical Cleanup Workflow</h2>

<p>Cleaning up dead code should be systematic, not heroic. Here is a workflow that teams can follow.</p>

<p><strong>Step 1: Baseline scan.</strong> Run <code>aigiscode analyze /path/to/project</code> to generate the initial report. Review the <code>dead_code</code> section of the JSON output.</p>

<p><strong>Step 2: Triage by confidence.</strong> Start with high-confidence findings. These are symbols that the deterministic analysis is certain are unused. Orphan files and unused imports with no dynamic reference patterns are typically high confidence.</p>

<p><strong>Step 3: Sample and verify.</strong> Before bulk-deleting, manually verify a sample of findings. Check that the code is truly unreachable. Search for dynamic references, configuration-based discovery, and reflection patterns that the static analysis might have missed.</p>

<p><strong>Step 4: Delete in small batches.</strong> Remove dead code in focused pull requests, one category or module at a time. This makes code review manageable and allows for easy reversion if a finding turns out to be a false positive.</p>

<p><strong>Step 5: Update policy.</strong> For false positives, add exclusion rules to <code>.aigiscode/rules.json</code> so they do not reappear in future scans. For patterns of false positives, encode them in <code>.aigiscode/policy.json</code> so the detector learns to skip similar patterns.</p>

<p><strong>Step 6: Prevent regression.</strong> Add AigisCode to your CI pipeline. Flag new dead code introductions in pull requests. The goal is not zero dead code but a consistent trend toward less of it.</p>

<h2 id="the-bottom-line">The Bottom Line</h2>

<p>Dead code is not an emergency. It does not cause production outages or data loss. But it is a slow, compounding tax on every engineering activity. It slows onboarding, inflates bundles, confuses developers, expands the security surface, and makes every other form of technical debt harder to address. The good news is that detecting it is a solved problem. The tools exist. The workflow is straightforward. The only barrier is making the decision to start.</p>
`,
      ar: `
<p>Dead code is the sediment of software development. Every feature that gets replaced, every experiment that does not ship, every refactoring that moves logic to a new location but forgets to remove the old one, they all leave behind code that is no longer executed. And unlike a broken test or a compile error, dead code does not announce its presence. It just sits there, quietly increasing the size, complexity, and maintenance burden of your codebase.</p>

<h2 id="how-dead-code-accumulates">How Dead Code Accumulates</h2>

<p>Dead code rarely appears through deliberate action. No developer writes a function with the intent of never calling it. Instead, dead code emerges through the natural lifecycle of software development.</p>

<p><strong>Feature replacement.</strong> A team builds version 2 of a feature, carefully routes all traffic to the new implementation, but never deletes the version 1 code because "we might need to roll back." Six months later, the rollback window has passed, but the old code remains because nobody remembers which files are safe to delete.</p>

<p><strong>Refactoring residue.</strong> A developer moves a utility function from <code>helpers.py</code> to <code>utils/string_helpers.py</code>. All call sites are updated. But the original function definition in <code>helpers.py</code> is left in place because the developer was not certain nothing else referenced it. The IDE shows no direct callers, but what about dynamic imports? What about tests? The safer choice feels like leaving it alone.</p>

<p><strong>Copy-paste evolution.</strong> A method is copied from one class to another to make a quick fix. The original is never removed. Over time, both copies evolve independently, and developers are unsure which one is the "real" version.</p>

<p><strong>Configuration drift.</strong> Environment variables, feature flags, and config keys accumulate over multiple release cycles. Old flags for features that shipped months ago remain in the config files, and the code that checks them remains in the application, even though the flag is always set to the same value.</p>

<h2 id="the-real-costs">The Real Costs of Dead Code</h2>

<h3>Developer Confusion</h3>

<p>The most immediate cost of dead code is cognitive load. A new developer joins the team, opens the codebase, and encounters a module with 15 classes. Five of them are dead. But the new developer does not know that. They spend time reading, understanding, and forming mental models of code that does not matter. Worse, they might build new features on top of dead code, creating dependencies on zombie implementations that nobody maintains.</p>

<p>This is not a theoretical concern. A 2025 survey of 1,200 developers by JetBrains found that <strong>63% of respondents</strong> cited "understanding unfamiliar code" as their biggest productivity bottleneck. Dead code directly inflates the amount of unfamiliar code that developers must navigate.</p>

<h3>Larger Bundles and Slower Builds</h3>

<p>In frontend applications, dead code directly impacts user experience. Unused components, abandoned utility functions, and orphan modules all get bundled into production JavaScript if tree-shaking fails to eliminate them. And tree-shaking has limits. If a module has side effects, or if the dead code is referenced through dynamic imports that the bundler cannot statically analyze, it ends up in the bundle.</p>

<p>On the backend, dead code increases build times, test execution time, and container image sizes. Every dead file is another file that the test runner must discover and skip. Every dead import is another module that must be resolved during startup.</p>

<h3>Expanded Security Surface</h3>

<p>Dead code that is still present in the repository is still present in production. If that dead code contains a vulnerability, such as an old API client that uses an insecure authentication method, or a deprecated utility that performs unsanitized string interpolation, the vulnerability is exploitable even if no active code path reaches it. An attacker who gains the ability to call arbitrary functions does not care whether those functions are "dead" from the application's perspective.</p>

<h3>CI and Tooling Overhead</h3>

<p>Every dead file adds to the workload of your continuous integration pipeline. Linters analyze it. Type checkers process it. Code coverage tools report on it, showing artificially low coverage percentages because dead code has zero test coverage by definition. Security scanners flag vulnerabilities in it. And developers spend time triaging these findings, only to discover they are about code that nobody uses.</p>

<h2 id="types-of-dead-code">Types of Dead Code</h2>

<p>Dead code is not monolithic. AigisCode detects several distinct categories, each with different detection strategies and risk profiles.</p>

<p><strong>Unused imports</strong> are the most common and easiest to detect. A module imports a symbol that is never used in the rest of the file. Most linters catch these at the single-file level, but cross-module unused imports, where a module re-exports a symbol that nobody else imports, require codebase-wide analysis.</p>

<p><strong>Unreferenced methods</strong> are class methods that are defined but never called from anywhere in the codebase. These are harder to detect because methods might be called dynamically through reflection, decorators, or framework conventions. AigisCode uses confidence levels to distinguish between methods that are definitively unused and those that might be called through dynamic dispatch.</p>

<p><strong>Abandoned classes</strong> are entire classes that are never instantiated or referenced. These often result from feature replacement where the new implementation uses a different class name or architecture.</p>

<p><strong>Orphan files</strong> are files with no inbound dependencies. No other file imports from them, no configuration references them, and no test targets them. These are often the clearest indicators of dead code because a file with no inbound dependencies can be safely deleted without affecting any other part of the codebase.</p>

<p><strong>Orphan properties</strong> are class attributes or object properties that are assigned but never read. These accumulate as data models evolve and fields are added for features that are later abandoned.</p>

<h2 id="detection-strategies">Detection Strategies</h2>

<p>Detecting dead code reliably requires cross-file analysis. A function that appears unused in its own file might be the only export of a utility module that 30 other files depend on. Conversely, a function that is imported in another file might only be imported for type-checking purposes and never actually called at runtime.</p>

<p>AigisCode approaches this by building a complete dependency graph from tree-sitter AST analysis. Every import, every function call, every class instantiation is tracked across the entire codebase. The dead code detector then identifies symbols that have no inbound references in the graph, flagging them with confidence levels based on how certain the analysis is.</p>

<p>For frameworks with convention-based code discovery, like Django's view functions referenced in URL configs or Laravel's service providers registered in config files, AigisCode's policy system allows teams to mark these entry points explicitly, preventing false positives without disabling the detector entirely.</p>

<h2 id="practical-cleanup-workflow">A Practical Cleanup Workflow</h2>

<p>Cleaning up dead code should be systematic, not heroic. Here is a workflow that teams can follow.</p>

<p><strong>Step 1: Baseline scan.</strong> Run <code>aigiscode analyze /path/to/project</code> to generate the initial report. Review the <code>dead_code</code> section of the JSON output.</p>

<p><strong>Step 2: Triage by confidence.</strong> Start with high-confidence findings. These are symbols that the deterministic analysis is certain are unused. Orphan files and unused imports with no dynamic reference patterns are typically high confidence.</p>

<p><strong>Step 3: Sample and verify.</strong> Before bulk-deleting, manually verify a sample of findings. Check that the code is truly unreachable. Search for dynamic references, configuration-based discovery, and reflection patterns that the static analysis might have missed.</p>

<p><strong>Step 4: Delete in small batches.</strong> Remove dead code in focused pull requests, one category or module at a time. This makes code review manageable and allows for easy reversion if a finding turns out to be a false positive.</p>

<p><strong>Step 5: Update policy.</strong> For false positives, add exclusion rules to <code>.aigiscode/rules.json</code> so they do not reappear in future scans. For patterns of false positives, encode them in <code>.aigiscode/policy.json</code> so the detector learns to skip similar patterns.</p>

<p><strong>Step 6: Prevent regression.</strong> Add AigisCode to your CI pipeline. Flag new dead code introductions in pull requests. The goal is not zero dead code but a consistent trend toward less of it.</p>

<h2 id="the-bottom-line">The Bottom Line</h2>

<p>Dead code is not an emergency. It does not cause production outages or data loss. But it is a slow, compounding tax on every engineering activity. It slows onboarding, inflates bundles, confuses developers, expands the security surface, and makes every other form of technical debt harder to address. The good news is that detecting it is a solved problem. The tools exist. The workflow is straightforward. The only barrier is making the decision to start.</p>
`,
      pl: `<h2 id="dead-code">Martwy kod i dług techniczny</h2>
<p>Martwy kod gromadzi się po cichu. AigisCode analizuje cały graf zależności, aby znaleźć kod bez odwołań.</p>`,
      bn: `
<p>ডেড কোড হলো সফটওয়্যার ডেভেলপমেন্টের পলি। প্রতিটি ফিচার যা প্রতিস্থাপিত হয়, প্রতিটি পরীক্ষা-নিরীক্ষা যা শিপ হয় না, প্রতিটি রিফ্যাক্টরিং যা লজিক নতুন জায়গায় সরায় কিন্তু পুরোনোটি মুছতে ভুলে যায় — এগুলো সবই এমন কোড রেখে যায় যা আর এক্সিকিউট হয় না। এবং একটি ভাঙা টেস্ট বা কম্পাইল এরর থেকে ভিন্ন, ডেড কোড তার উপস্থিতি ঘোষণা করে না। এটি শুধু সেখানে বসে থাকে, নিরবে আপনার কোডবেসের আকার, জটিলতা এবং রক্ষণাবেক্ষণের বোঝা বাড়ায়।</p>

<h2 id="how-dead-code-accumulates">ডেড কোড কিভাবে জমা হয়</h2>

<p>ডেড কোড খুব কমই ইচ্ছাকৃত কর্মের মাধ্যমে দেখা দেয়। কোনো ডেভেলপার কখনো কল না করার উদ্দেশ্যে একটি ফাংশন লেখে না। বরং, সফটওয়্যার ডেভেলপমেন্টের স্বাভাবিক জীবনচক্রের মধ্য দিয়ে ডেড কোড উদ্ভূত হয়।</p>

<p><strong>ফিচার প্রতিস্থাপন।</strong> একটি টিম একটি ফিচারের ভার্সন ২ তৈরি করে, সাবধানে সব ট্র্যাফিক নতুন ইমপ্লিমেন্টেশনে রুট করে, কিন্তু কখনো ভার্সন ১-এর কোড মুছে না কারণ "আমাদের রোলব্যাক করতে হতে পারে।" ছয় মাস পরে, রোলব্যাক উইন্ডো পার হয়ে গেছে, কিন্তু পুরোনো কোড থেকে যায় কারণ কেউ মনে করতে পারে না কোন ফাইলগুলো নিরাপদে মুছে ফেলা যায়।</p>

<p><strong>রিফ্যাক্টরিং অবশিষ্টাংশ।</strong> একজন ডেভেলপার একটি ইউটিলিটি ফাংশন <code>helpers.py</code> থেকে <code>utils/string_helpers.py</code>-তে সরান। সব কল সাইট আপডেট করা হয়। কিন্তু <code>helpers.py</code>-তে মূল ফাংশন সংজ্ঞাটি রেখে দেওয়া হয় কারণ ডেভেলপার নিশ্চিত ছিলেন না যে অন্য কিছু এটি রেফারেন্স করছে না। IDE কোনো সরাসরি কলার দেখায় না, কিন্তু ডায়নামিক ইমপোর্ট সম্পর্কে কী? টেস্ট সম্পর্কে কী? এটি রেখে দেওয়াই নিরাপদ মনে হয়।</p>

<p><strong>কপি-পেস্ট বিবর্তন।</strong> দ্রুত ফিক্স করতে একটি মেথড এক ক্লাস থেকে অন্য ক্লাসে কপি করা হয়। মূলটি কখনো সরানো হয় না। সময়ের সাথে, উভয় কপি স্বাধীনভাবে বিবর্তিত হয়, এবং ডেভেলপাররা অনিশ্চিত থাকে কোনটি "আসল" ভার্সন।</p>

<p><strong>কনফিগারেশন ড্রিফ্ট।</strong> এনভায়রনমেন্ট ভেরিয়েবল, ফিচার ফ্ল্যাগ এবং কনফিগ কী একাধিক রিলিজ সাইকেলে জমা হয়। মাসে আগে শিপ হওয়া ফিচারগুলোর পুরোনো ফ্ল্যাগগুলো কনফিগ ফাইলে থেকে যায়, এবং সেগুলো চেক করা কোড অ্যাপ্লিকেশনে থেকে যায়, যদিও ফ্ল্যাগটি সর্বদা একই মানে সেট করা থাকে।</p>

<h2 id="the-real-costs">ডেড কোডের আসল খরচ</h2>

<h3>ডেভেলপার বিভ্রান্তি</h3>

<p>ডেড কোডের সবচেয়ে তাৎক্ষণিক খরচ হলো কগনিটিভ লোড। একজন নতুন ডেভেলপার টিমে যোগ দেয়, কোডবেস খোলে, এবং ১৫টি ক্লাস সহ একটি মডিউলের সম্মুখীন হয়। পাঁচটি ডেড। কিন্তু নতুন ডেভেলপার সেটা জানে না। তারা এমন কোড পড়তে, বুঝতে এবং মানসিক মডেল তৈরি করতে সময় ব্যয় করে যা গুরুত্বপূর্ণ নয়। আরও খারাপ, তারা ডেড কোডের উপর নতুন ফিচার তৈরি করতে পারে, জম্বি ইমপ্লিমেন্টেশনের উপর ডিপেন্ডেন্সি তৈরি করে যা কেউ রক্ষণাবেক্ষণ করে না।</p>

<p>এটি তাত্ত্বিক উদ্বেগ নয়। JetBrains-এর ১,২০০ ডেভেলপারের ২০২৫ সালের একটি সমীক্ষায় দেখা গেছে যে <strong>৬৩% উত্তরদাতা</strong> "অপরিচিত কোড বোঝা"-কে তাদের সবচেয়ে বড় উৎপাদনশীলতার বাধা হিসেবে উল্লেখ করেছে। ডেড কোড সরাসরি ডেভেলপারদের নেভিগেট করতে হওয়া অপরিচিত কোডের পরিমাণ বাড়ায়।</p>

<h3>বড় বান্ডেল এবং ধীর বিল্ড</h3>

<p>ফ্রন্টএন্ড অ্যাপ্লিকেশনে, ডেড কোড সরাসরি ব্যবহারকারীর অভিজ্ঞতাকে প্রভাবিত করে। অব্যবহৃত কম্পোনেন্ট, পরিত্যক্ত ইউটিলিটি ফাংশন এবং অনাথ মডিউলগুলো tree-shaking তাদের দূর করতে ব্যর্থ হলে প্রোডাকশন JavaScript-এ বান্ডেল হয়ে যায়। এবং tree-shaking-এর সীমাবদ্ধতা আছে। যদি একটি মডিউলের সাইড ইফেক্ট থাকে, বা যদি ডেড কোড ডায়নামিক ইমপোর্টের মাধ্যমে রেফারেন্স করা হয় যা বান্ডলার স্ট্যাটিক্যালি বিশ্লেষণ করতে পারে না, এটি বান্ডেলে চলে আসে।</p>

<p>ব্যাকএন্ডে, ডেড কোড বিল্ড টাইম, টেস্ট এক্সিকিউশন টাইম এবং কন্টেইনার ইমেজ সাইজ বাড়ায়। প্রতিটি ডেড ফাইল আরেকটি ফাইল যা টেস্ট রানারকে খুঁজে বের করতে এবং এড়িয়ে যেতে হবে। প্রতিটি ডেড ইমপোর্ট আরেকটি মডিউল যা স্টার্টআপের সময় রিজলভ করতে হবে।</p>

<h3>বিস্তৃত সিকিউরিটি সারফেস</h3>

<p>রিপোজিটরিতে উপস্থিত ডেড কোড এখনও প্রোডাকশনে উপস্থিত। যদি সেই ডেড কোডে একটি ভালনারেবিলিটি থাকে, যেমন একটি পুরোনো API ক্লায়েন্ট যা একটি অনিরাপদ অথেন্টিকেশন পদ্ধতি ব্যবহার করে, বা একটি ডেপ্রিকেটেড ইউটিলিটি যা আনস্যানিটাইজড স্ট্রিং ইন্টারপোলেশন করে, ভালনারেবিলিটি এক্সপ্লয়টযোগ্য এমনকি কোনো সক্রিয় কোড পাথ এটিতে না পৌঁছালেও। একজন আক্রমণকারী যে আর্বিট্রারি ফাংশন কল করার ক্ষমতা অর্জন করে সে সেই ফাংশনগুলো অ্যাপ্লিকেশনের দৃষ্টিকোণ থেকে "ডেড" কিনা তা নিয়ে চিন্তা করে না।</p>

<h3>CI এবং টুলিং ওভারহেড</h3>

<p>প্রতিটি ডেড ফাইল আপনার continuous integration পাইপলাইনের কাজের চাপ বাড়ায়। লিন্টারগুলো এটি বিশ্লেষণ করে। টাইপ চেকারগুলো এটি প্রসেস করে। কোড কভারেজ টুলগুলো এটির রিপোর্ট করে, কৃত্রিমভাবে কম কভারেজ শতাংশ দেখায় কারণ ডেড কোডের সংজ্ঞা অনুযায়ী জিরো টেস্ট কভারেজ থাকে। সিকিউরিটি স্ক্যানারগুলো এতে ভালনারেবিলিটি ফ্ল্যাগ করে। এবং ডেভেলপাররা এই ফলাফলগুলো ট্রায়াজ করতে সময় ব্যয় করে, শুধু আবিষ্কার করতে যে এগুলো এমন কোড সম্পর্কে যা কেউ ব্যবহার করে না।</p>

<h2 id="types-of-dead-code">ডেড কোডের প্রকারভেদ</h2>

<p>ডেড কোড একচেটিয়া নয়। AigisCode বিভিন্ন স্বতন্ত্র বিভাগ শনাক্ত করে, প্রতিটির ভিন্ন শনাক্তকরণ কৌশল এবং ঝুঁকি প্রোফাইল রয়েছে।</p>

<p><strong>অব্যবহৃত ইমপোর্ট</strong> সবচেয়ে সাধারণ এবং শনাক্ত করা সবচেয়ে সহজ। একটি মডিউল একটি সিম্বল ইমপোর্ট করে যা ফাইলের বাকি অংশে কখনো ব্যবহৃত হয় না। বেশিরভাগ লিন্টার একক-ফাইল স্তরে এগুলো ধরে, কিন্তু ক্রস-মডিউল অব্যবহৃত ইমপোর্ট, যেখানে একটি মডিউল একটি সিম্বল রি-এক্সপোর্ট করে যা অন্য কেউ ইমপোর্ট করে না, কোডবেস-ব্যাপী বিশ্লেষণ প্রয়োজন।</p>

<p><strong>অরেফারেন্সড মেথড</strong> হলো ক্লাস মেথড যা সংজ্ঞায়িত কিন্তু কোডবেসের কোথাও থেকে কখনো কল করা হয় না। এগুলো শনাক্ত করা কঠিন কারণ মেথড রিফ্লেকশন, ডেকোরেটর, বা ফ্রেমওয়ার্ক কনভেনশনের মাধ্যমে ডায়নামিক্যালি কল হতে পারে। AigisCode নিশ্চিতভাবে অব্যবহৃত এবং ডায়নামিক ডিসপ্যাচের মাধ্যমে কল হতে পারে এমন মেথডের মধ্যে পার্থক্য করতে কনফিডেন্স লেভেল ব্যবহার করে।</p>

<p><strong>পরিত্যক্ত ক্লাস</strong> হলো সম্পূর্ণ ক্লাস যা কখনো ইনস্ট্যান্সিয়েট বা রেফারেন্স করা হয় না। এগুলো প্রায়ই ফিচার প্রতিস্থাপনের ফলে হয় যেখানে নতুন ইমপ্লিমেন্টেশন একটি ভিন্ন ক্লাস নাম বা আর্কিটেকচার ব্যবহার করে।</p>

<p><strong>অনাথ ফাইল</strong> হলো কোনো ইনবাউন্ড ডিপেন্ডেন্সি নেই এমন ফাইল। অন্য কোনো ফাইল এগুলো থেকে ইমপোর্ট করে না, কোনো কনফিগারেশন এগুলো রেফারেন্স করে না, এবং কোনো টেস্ট এগুলো টার্গেট করে না। এগুলো প্রায়ই ডেড কোডের সবচেয়ে স্পষ্ট সূচক কারণ কোনো ইনবাউন্ড ডিপেন্ডেন্সি নেই এমন একটি ফাইল কোডবেসের অন্য কোনো অংশকে প্রভাবিত না করে নিরাপদে মুছে ফেলা যায়।</p>

<p><strong>অনাথ প্রপার্টি</strong> হলো ক্লাস অ্যাট্রিবিউট বা অবজেক্ট প্রপার্টি যা অ্যাসাইন করা হয় কিন্তু কখনো পড়া হয় না। ডেটা মডেল বিবর্তিত হওয়ার সাথে সাথে এবং পরে পরিত্যক্ত ফিচারের জন্য ফিল্ড যোগ করার সাথে সাথে এগুলো জমা হয়।</p>

<h2 id="detection-strategies">শনাক্তকরণ কৌশল</h2>

<p>নির্ভরযোগ্যভাবে ডেড কোড শনাক্ত করতে ক্রস-ফাইল বিশ্লেষণ প্রয়োজন। একটি ফাংশন যা তার নিজের ফাইলে অব্যবহৃত দেখায় তা একটি ইউটিলিটি মডিউলের একমাত্র এক্সপোর্ট হতে পারে যার উপর ৩০টি অন্য ফাইল নির্ভর করে। বিপরীতভাবে, একটি ফাংশন যা অন্য ফাইলে ইমপোর্ট করা হয়েছে তা শুধুমাত্র টাইপ-চেকিং উদ্দেশ্যে ইমপোর্ট করা হতে পারে এবং রানটাইমে কখনো আসলে কল করা হয় না।</p>

<p>AigisCode tree-sitter AST বিশ্লেষণ থেকে একটি সম্পূর্ণ ডিপেন্ডেন্সি গ্রাফ তৈরি করে এটি সমাধান করে। প্রতিটি ইমপোর্ট, প্রতিটি ফাংশন কল, প্রতিটি ক্লাস ইনস্ট্যান্সিয়েশন সম্পূর্ণ কোডবেস জুড়ে ট্র্যাক করা হয়। ডেড কোড ডিটেক্টর তারপর গ্রাফে কোনো ইনবাউন্ড রেফারেন্স নেই এমন সিম্বলগুলো চিহ্নিত করে, বিশ্লেষণ কতটা নিশ্চিত তার উপর ভিত্তি করে কনফিডেন্স লেভেল সহ ফ্ল্যাগ করে।</p>

<p>কনভেনশন-ভিত্তিক কোড ডিসকভারি সহ ফ্রেমওয়ার্কের জন্য, যেমন URL কনফিগে রেফারেন্সকৃত Django-র ভিউ ফাংশন বা কনফিগ ফাইলে রেজিস্টার্ড Laravel-এর সার্ভিস প্রোভাইডার, AigisCode-এর পলিসি সিস্টেম টিমদের এই এন্ট্রি পয়েন্টগুলো স্পষ্টভাবে চিহ্নিত করতে দেয়, ডিটেক্টর সম্পূর্ণ নিষ্ক্রিয় না করেই false positive প্রতিরোধ করে।</p>

<h2 id="practical-cleanup-workflow">একটি ব্যবহারিক ক্লিনআপ ওয়ার্কফ্লো</h2>

<p>ডেড কোড পরিষ্কার করা পদ্ধতিগত হওয়া উচিত, বীরত্বপূর্ণ নয়। এখানে একটি ওয়ার্কফ্লো যা টিমগুলো অনুসরণ করতে পারে।</p>

<p><strong>ধাপ ১: বেসলাইন স্ক্যান।</strong> প্রাথমিক রিপোর্ট তৈরি করতে <code>aigiscode analyze /path/to/project</code> চালান। JSON আউটপুটের <code>dead_code</code> সেকশন রিভিউ করুন।</p>

<p><strong>ধাপ ২: কনফিডেন্স অনুযায়ী ট্রায়াজ।</strong> উচ্চ-কনফিডেন্স ফলাফল দিয়ে শুরু করুন। এগুলো এমন সিম্বল যা ডিটারমিনিস্টিক বিশ্লেষণ নিশ্চিত যে অব্যবহৃত। অনাথ ফাইল এবং কোনো ডায়নামিক রেফারেন্স প্যাটার্ন ছাড়া অব্যবহৃত ইমপোর্ট সাধারণত উচ্চ কনফিডেন্স।</p>

<p><strong>ধাপ ৩: নমুনা এবং যাচাই।</strong> গণ-মুছে ফেলার আগে, ফলাফলের একটি নমুনা ম্যানুয়ালি যাচাই করুন। কোডটি সত্যিই অপ্রাপ্য কিনা পরীক্ষা করুন। ডায়নামিক রেফারেন্স, কনফিগারেশন-ভিত্তিক ডিসকভারি, এবং রিফ্লেকশন প্যাটার্ন খুঁজুন যা স্ট্যাটিক বিশ্লেষণ মিস করতে পারে।</p>

<p><strong>ধাপ ৪: ছোট ব্যাচে মুছুন।</strong> ফোকাসড পুল রিকোয়েস্টে ডেড কোড সরান, একবারে একটি বিভাগ বা মডিউল। এটি কোড রিভিউ পরিচালনাযোগ্য করে এবং যদি একটি ফলাফল false positive হয় তবে সহজ রিভার্সনের সুযোগ দেয়।</p>

<p><strong>ধাপ ৫: পলিসি আপডেট।</strong> false positive-এর জন্য, <code>.aigiscode/rules.json</code>-এ এক্সক্লুশন নিয়ম যোগ করুন যাতে ভবিষ্যত স্ক্যানে এগুলো পুনরায় দেখা না দেয়। false positive-এর প্যাটার্নের জন্য, <code>.aigiscode/policy.json</code>-এ এনকোড করুন যাতে ডিটেক্টর অনুরূপ প্যাটার্ন এড়িয়ে যেতে শেখে।</p>

<p><strong>ধাপ ৬: রিগ্রেশন প্রতিরোধ।</strong> আপনার CI পাইপলাইনে AigisCode যোগ করুন। পুল রিকোয়েস্টে নতুন ডেড কোড পরিচয় ফ্ল্যাগ করুন। লক্ষ্য জিরো ডেড কোড নয় বরং এটি কমানোর একটি ধারাবাহিক প্রবণতা।</p>

<h2 id="the-bottom-line">মূল কথা</h2>

<p>ডেড কোড একটি জরুরি অবস্থা নয়। এটি প্রোডাকশন আউটেজ বা ডেটা ক্ষতি ঘটায় না। কিন্তু এটি প্রতিটি ইঞ্জিনিয়ারিং কার্যকলাপে একটি ধীর, সংযুক্ত কর। এটি অনবোর্ডিং ধীর করে, বান্ডেল ফুলিয়ে তোলে, ডেভেলপারদের বিভ্রান্ত করে, সিকিউরিটি সারফেস বিস্তৃত করে এবং অন্য প্রতিটি ধরনের টেকনিক্যাল ডেট সমাধান করা কঠিন করে তোলে। ভালো খবর হলো এটি শনাক্ত করা একটি সমাধানকৃত সমস্যা। টুলগুলো বিদ্যমান। ওয়ার্কফ্লো সরল। একমাত্র বাধা হলো শুরু করার সিদ্ধান্ত নেওয়া।</p>
`,
    },
  },

  /* ======================================================================== */
  /*  4. Static Analysis vs Linters: What You Actually Need in 2026           */
  /* ======================================================================== */
  {
    slug: 'static-analysis-vs-linters-2026',
    date: '2026-01-14',
    readTime: 8,
    tags: ['Static Analysis', 'Linting', 'Code Quality', 'Comparison'],
    image: '/blog-static-analysis-vs-linters.jpg',
    author: { name: 'David Strejc', role: 'Creator of AigisCode' },
    relatedSlugs: [
      'why-ai-code-analysis-matters-2026',
      'dead-code-technical-debt',
    ],
    title: {
      en: 'Static Analysis vs Linters: What You Actually Need in 2026',
      cs: 'Statická analýza vs lintery: Co skutečně potřebujete v roce 2026',
      fr: 'Analyse statique vs linters : ce dont vous avez vraiment besoin en 2026',
      es: 'Analisis estatico vs linters: lo que realmente necesitas en 2026',
      zh: '静态分析 vs Linter：2026 年你真正需要什么',
      hi: 'स्टैटिक एनालिसिस बनाम लिंटर: 2026 में आपको वास्तव में क्या चाहिए',
      pt: 'Análise estática vs linters: o que você realmente precisa em 2026',
      ar: 'التحليل الثابت مقابل أدوات الفحص: ما تحتاجه فعلاً في 2026',
      pl: 'Analiza statyczna vs lintery: czego naprawdę potrzebujesz w 2026 roku',
      bn: 'স্ট্যাটিক অ্যানালিসিস বনাম লিন্টার: ২০২৬ সালে আপনার আসলে কী দরকার',
    },
    description: {
      en: 'ESLint, Pylint, and PHPStan check files one at a time. Static analysis tools like AigisCode analyze your entire codebase as a graph. Here is what each catches, what they miss, and when you need both.',
      cs: 'ESLint a Pylint kontrolují soubory jednotlivě. Nástroje statické analýzy analyzují celý codebase jako graf.',
      fr: 'ESLint et Pylint verifient les fichiers un par un. Les outils d\'analyse statique analysent votre codebase comme un graphe.',
      es: 'ESLint y Pylint verifican archivos uno a la vez. Las herramientas de analisis estatico analizan tu codigo como un grafo.',
      zh: 'ESLint 和 Pylint 逐个检查文件。静态分析工具将整个代码库作为图来分析。',
      hi: 'ESLint और Pylint फ़ाइलों को एक-एक करके जाँचते हैं। स्टैटिक एनालिसिस टूल पूरे कोडबेस को ग्राफ़ के रूप में विश्लेषण करते हैं।',
      pt: 'ESLint e Pylint verificam arquivos um de cada vez. Ferramentas de análise estática analisam seu código como um grafo.',
      ar: 'يفحص ESLint وPylint وPHPStan الملفات واحداً تلو الآخر. تحلل أدوات التحليل الثابت مثل AigisCode قاعدة شيفرتك بالكامل كرسم بياني. إليك ما يكتشفه كل منها وما يفوته ومتى تحتاج كليهما.',
      pl: 'ESLint, Pylint i PHPStan sprawdzają pliki pojedynczo. Narzędzia analizy statycznej, takie jak AigisCode, analizują cały graf zależności. Dowiedz się, kiedy potrzebujesz każdego z nich — i dlaczego odpowiedź to zwykle oba.',
      bn: 'ESLint, Pylint এবং PHPStan একবারে একটি ফাইল চেক করে। AigisCode-এর মতো স্ট্যাটিক অ্যানালিসিস টুলস আপনার সম্পূর্ণ কোডবেস গ্রাফ হিসেবে বিশ্লেষণ করে। এখানে প্রতিটি কী ধরে, কী মিস করে এবং কখন উভয়ই দরকার।',
    },
    metaDescription: {
      en: 'Compare static analysis tools with linters like ESLint, Pylint, and PHPStan. Learn what each catches, what they miss, and how AigisCode complements your existing linting setup.',
      cs: 'Porovnejte nástroje statické analýzy s lintery jako ESLint a Pylint. Zjistěte, co každý z nich zachytí.',
      fr: 'Comparez les outils d\'analyse statique avec les linters comme ESLint et Pylint. Decouvrez ce que chacun detecte.',
      es: 'Compare herramientas de analisis estatico con linters como ESLint y Pylint. Descubra que detecta cada uno.',
      zh: '将静态分析工具与 ESLint、Pylint 等 linter 进行比较。了解各自的检测能力。',
      hi: 'ESLint और Pylint जैसे लिंटर्स के साथ स्टैटिक एनालिसिस टूल की तुलना करें।',
      pt: 'Compare ferramentas de análise estática com linters como ESLint e Pylint. Saiba o que cada um detecta.',
      ar: 'قارن أدوات التحليل الثابت مع أدوات الفحص مثل ESLint وPylint وPHPStan. تعرّف ما يكتشفه كل منها وما يفوته وكيف يكمّل AigisCode إعداد الفحص الحالي لديك.',
      pl: 'Porównaj narzędzia analizy statycznej z linterami takimi jak ESLint, Pylint i PHPStan. Dowiedz się, co każde z nich wykrywa, co pomijają i jak AigisCode uzupełnia Twoją istniejącą konfigurację lintingu.',
      bn: 'ESLint, Pylint এবং PHPStan-এর মতো লিন্টারের সাথে স্ট্যাটিক অ্যানালিসিস টুলসের তুলনা করুন। প্রতিটি কী ধরে, কী মিস করে এবং AigisCode কিভাবে আপনার বিদ্যমান লিন্টিং সেটআপের পরিপূরক তা জানুন।',
    },
    content: {
      en: `
<p>If you have worked on a professional software project in the last decade, you have used a linter. ESLint for JavaScript, Pylint or Ruff for Python, PHPStan or Psalm for PHP, Clippy for Rust. Linters are ubiquitous, well-understood, and indispensable. They catch bugs, enforce style, and maintain consistency across teams. So why would anyone need something beyond a linter?</p>

<p>The answer lies in scope. Linters analyze files. Static analysis tools analyze codebases. The difference sounds subtle, but it changes everything about what problems you can detect.</p>

<h2 id="what-linters-do-well">What Linters Do Well</h2>

<p>Linters excel at <strong>intra-file analysis</strong>. Within a single file, a linter can detect unused variables, unreachable code branches, type mismatches, style violations, potential null dereferences, and dozens of other issues. Modern linters are remarkably sophisticated. TypeScript's built-in type checker performs deep flow analysis within functions. PHPStan at level 9 catches subtle type-narrowing issues that would be invisible to a human reviewer. Ruff can check 500 Python lint rules in under a second.</p>

<p>Linters also integrate seamlessly into the developer workflow. They run in your editor, providing real-time feedback as you type. They run in CI, blocking merges that introduce violations. They are fast, incremental, and deterministic. For file-level code quality, linters are the right tool.</p>

<h2 id="what-linters-miss">What Linters Miss</h2>

<p>The fundamental limitation of linters is that they process files independently. They do not build a graph of how your files relate to each other. This means they cannot detect an entire category of problems that only become visible when you look at the codebase as a connected system.</p>

<h3>Circular Dependencies</h3>

<p>A linter can see that <code>orders.py</code> imports from <code>inventory.py</code>. It can verify that the imported symbol exists and has the right type. What it cannot see is that <code>inventory.py</code> also imports from <code>orders.py</code>, creating a cycle. And it certainly cannot see that this cycle is part of a larger four-module loop that makes the entire order management subsystem impossible to test or deploy independently.</p>

<p>Detecting circular dependencies requires building a dependency graph of the entire codebase and running cycle-detection algorithms on it. This is fundamentally a codebase-level operation, not a file-level one.</p>

<h3>Cross-File Dead Code</h3>

<p>A linter can tell you that a variable is unused within a file. But it cannot tell you that an exported function is never imported by any other file in the project. It cannot tell you that an entire module has no inbound dependencies and is effectively orphaned. It cannot tell you that a class method is defined but never called from anywhere in the codebase.</p>

<p>Cross-file dead code detection requires knowing the complete import graph. Which modules import which symbols? Which class methods are called and from where? Which files are entry points and which are libraries? These questions cannot be answered by examining files in isolation.</p>

<h3>Architectural Violations</h3>

<p>Many teams define architectural rules, even if only informally. "Controllers should not import from other controllers." "The data layer should not depend on the presentation layer." "Utility modules should not import from feature modules." Linters cannot enforce these rules because they do not know about the layered structure of your codebase. They see individual files, not the dependency hierarchy between packages.</p>

<h3>God Classes and Bottleneck Files</h3>

<p>A god class, a class that has grown to handle too many responsibilities, is a well-known code smell. But measuring it requires more than counting lines of code. A true god class analysis looks at how many other modules depend on the class, how many different concerns it addresses, and whether its responsibilities can be separated without creating circular dependencies. Similarly, a bottleneck file, one that sits on too many dependency paths, can only be identified by analyzing the graph structure of the codebase.</p>

<h2 id="what-static-analysis-adds">What Static Analysis Adds</h2>

<p>Static analysis tools like AigisCode operate at the codebase level. They parse every file, extract symbols and dependencies, build a graph, and then run analysis algorithms on that graph. This enables an entirely different class of detections.</p>

<table>
<thead>
<tr><th>Capability</th><th>Linter</th><th>Static Analysis</th></tr>
</thead>
<tbody>
<tr><td>Unused variables in a file</td><td>Yes</td><td>Not the focus</td></tr>
<tr><td>Style enforcement</td><td>Yes</td><td>No</td></tr>
<tr><td>Type checking within functions</td><td>Yes</td><td>No</td></tr>
<tr><td>Circular dependencies</td><td>No</td><td>Yes</td></tr>
<tr><td>Cross-file dead code</td><td>No</td><td>Yes</td></tr>
<tr><td>Orphan files / modules</td><td>No</td><td>Yes</td></tr>
<tr><td>God classes (by coupling)</td><td>No</td><td>Yes</td></tr>
<tr><td>Bottleneck files</td><td>No</td><td>Yes</td></tr>
<tr><td>Layer violations</td><td>Limited</td><td>Yes</td></tr>
<tr><td>Hardwired values across files</td><td>Limited</td><td>Yes</td></tr>
</tbody>
</table>

<p>The key insight is that these tools are <strong>complementary</strong>, not competing. A linter ensures each brick is well-formed. Static analysis ensures the building stands straight.</p>

<h2 id="when-you-need-both">When You Need Both</h2>

<p>Every professional project should have a linter. The question is when you also need codebase-level static analysis. Here are the signals.</p>

<p><strong>Your codebase has more than 100 files.</strong> Below this threshold, most architectural issues are visible through casual reading. Above it, the dependency graph becomes too complex to hold in your head, and hidden cycles and dead code begin to accumulate.</p>

<p><strong>Multiple developers contribute to the project.</strong> A solo developer has perfect context about which code is active and which is dead. A team of five developers, working in parallel, will inevitably create dead code and accidental cycles because no single person has full visibility of every change.</p>

<p><strong>The project has been active for more than six months.</strong> Technical debt is a function of time. The longer a codebase lives, the more dead code, architectural drift, and hidden coupling it accumulates. Static analysis provides the periodic health check that catches these slow-moving issues.</p>

<p><strong>You are preparing for a major refactoring.</strong> Before restructuring a codebase, you need to understand its current shape. Which modules are tightly coupled? Where are the cycles? Which files can be moved independently? Static analysis gives you the map you need before you start making changes.</p>

<p><strong>AI agents are contributing code.</strong> AI coding agents generate code quickly but do not always have full context about the architectural norms of your project. Running static analysis on AI-generated contributions ensures they do not introduce new cycles or violate layering rules.</p>

<h2 id="how-aigiscode-complements-linters">How AigisCode Complements Your Linters</h2>

<p>AigisCode is designed to sit alongside your existing linting setup, not replace it. You keep ESLint for JavaScript style and type safety. You keep Pylint or Ruff for Python conventions. You add AigisCode for the cross-file, architectural analysis that linters cannot provide.</p>

<p>In practice, this means running your linter on every file change (in your editor and CI) and running AigisCode periodically or on pull requests that touch more than a handful of files. The JSON report integrates cleanly into CI workflows, and the policy system lets you tune detection sensitivity to match your project's norms.</p>

<p>The result is two complementary layers of code quality assurance. The linter catches the small issues immediately. The static analyzer catches the structural issues before they compound. Together, they cover the full spectrum of code quality, from individual statements to architectural integrity.</p>

<h2 id="the-right-tool-for-the-right-job">The Right Tool for the Right Job</h2>

<p>The linter versus static analysis debate is a false dichotomy. It is like asking whether you need a microscope or a telescope. They look at different scales. Linters look at the code in front of you. Static analysis looks at the shape of your entire codebase. In 2026, with codebases growing larger and more complex every quarter, you need both. The question is not "which one" but "when do I add the second one." For most teams, the answer is: sooner than you think.</p>
`,
      cs: `
<p>Pokud jste v posledním desetiletí pracovali na profesionálním softwarovém projektu, použili jste linter. ESLint pro JavaScript, Pylint nebo Ruff pro Python, PHPStan nebo Psalm pro PHP, Clippy pro Rust. Lintery jsou všudypřítomné, dobře pochopené a nepostradatelné. Zachytávají chyby, vynucují styl a udržují konzistenci napříč týmy. Tak proč by někdo potřeboval něco víc než linter?</p>

<p>Odpověď spočívá v rozsahu. Lintery analyzují soubory. Nástroje statické analýzy analyzují codebase. Rozdíl zní jemně, ale mění vše o tom, jaké problémy můžete detekovat.</p>

<h2 id="what-linters-do-well">V čem lintery vynikají</h2>

<p>Lintery vynikají v <strong>analýze uvnitř souboru</strong>. V rámci jednoho souboru dokáže linter detekovat nepoužívané proměnné, nedosažitelné větve kódu, nesoulad typů, porušení stylu, potenciální null dereference a desítky dalších problémů. Moderní lintery jsou pozoruhodně sofistikované. Vestavěný typový kontrolor TypeScriptu provádí hloubkovou flow analýzu uvnitř funkcí. PHPStan na úrovni 9 zachytí subtilní problémy se zúžením typů, které by byly pro lidského recenzenta neviditelné. Ruff dokáže zkontrolovat 500 pravidel pro Python lint za méně než sekundu.</p>

<p>Lintery se také bezproblémově integrují do pracovního postupu vývojáře. Běží ve vašem editoru a poskytují zpětnou vazbu v reálném čase při psaní. Běží v CI a blokují merge, které zavádějí porušení. Jsou rychlé, inkrementální a deterministické. Pro kvalitu kódu na úrovni souboru jsou lintery správný nástroj.</p>

<h2 id="what-linters-miss">Co lintery přehlížejí</h2>

<p>Zásadním omezením linterů je, že zpracovávají soubory nezávisle. Nebudují graf toho, jak spolu vaše soubory souvisejí. To znamená, že nedokáží detekovat celou kategorii problémů, které se stanou viditelnými teprve tehdy, když se na codebase podíváte jako na propojený systém.</p>

<h3>Cyklické závislosti</h3>

<p>Linter vidí, že <code>orders.py</code> importuje z <code>inventory.py</code>. Může ověřit, že importovaný symbol existuje a má správný typ. Co nevidí, je to, že <code>inventory.py</code> také importuje z <code>orders.py</code>, čímž vzniká cyklus. A rozhodně nevidí, že tento cyklus je součástí větší smyčky čtyř modulů, která znemožňuje nezávislé testování nebo nasazení celého subsystému správy objednávek.</p>

<p>Detekce cyklických závislostí vyžaduje budování grafu závislostí celého codebase a spouštění algoritmů detekce cyklů na něm. To je zásadně operace na úrovni codebase, nikoli na úrovni souboru.</p>

<h3>Cross-file mrtvý kód</h3>

<p>Linter vám může říct, že proměnná je nepoužívaná v rámci souboru. Ale nemůže vám říct, že exportovaná funkce není nikdy importována žádným jiným souborem v projektu. Nemůže vám říct, že celý modul nemá žádné příchozí závislosti a je fakticky osiřelý. Nemůže vám říct, že metoda třídy je definována, ale nikdy volána odkudkoli v codebase.</p>

<p>Detekce cross-file mrtvého kódu vyžaduje znalost kompletního importního grafu. Které moduly importují které symboly? Které metody tříd jsou volány a odkud? Které soubory jsou vstupní body a které jsou knihovny? Na tyto otázky nelze odpovědět zkoumáním souborů izolovaně.</p>

<h3>Architektonická porušení</h3>

<p>Mnoho týmů definuje architektonická pravidla, byť jen neformálně. „Kontrolery by neměly importovat z jiných kontrolerů." „Datová vrstva by neměla záviset na prezentační vrstvě." „Utilitní moduly by neměly importovat z feature modulů." Lintery tato pravidla nemohou vynucovat, protože nevědí o vrstvené struktuře vašeho codebase. Vidí jednotlivé soubory, nikoli hierarchii závislostí mezi balíčky.</p>

<h3>God třídy a úzká hrdla souborů</h3>

<p>God třída — třída, která narostla natolik, že zpracovává příliš mnoho odpovědností — je dobře známý code smell. Ale její měření vyžaduje více než počítání řádků kódu. Skutečná analýza god třídy zkoumá, kolik dalších modulů na třídě závisí, kolik různých záležitostí řeší a zda lze její odpovědnosti oddělit bez vytvoření cyklických závislostí. Podobně soubor úzkého hrdla — soubor na příliš mnoha cestách závislostí — lze identifikovat pouze analýzou grafové struktury codebase.</p>

<h2 id="what-static-analysis-adds">Co přidává statická analýza</h2>

<p>Nástroje statické analýzy jako AigisCode operují na úrovni codebase. Parsují každý soubor, extrahují symboly a závislosti, budují graf a poté na něm spouštějí analytické algoritmy. To umožňuje zcela odlišnou třídu detekcí.</p>

<table>
<thead>
<tr><th>Schopnost</th><th>Linter</th><th>Statická analýza</th></tr>
</thead>
<tbody>
<tr><td>Nepoužívané proměnné v souboru</td><td>Ano</td><td>Není zaměření</td></tr>
<tr><td>Vynucování stylu</td><td>Ano</td><td>Ne</td></tr>
<tr><td>Typová kontrola uvnitř funkcí</td><td>Ano</td><td>Ne</td></tr>
<tr><td>Cyklické závislosti</td><td>Ne</td><td>Ano</td></tr>
<tr><td>Cross-file mrtvý kód</td><td>Ne</td><td>Ano</td></tr>
<tr><td>Osiřelé soubory / moduly</td><td>Ne</td><td>Ano</td></tr>
<tr><td>God třídy (podle provázanosti)</td><td>Ne</td><td>Ano</td></tr>
<tr><td>Soubory úzkých hrdel</td><td>Ne</td><td>Ano</td></tr>
<tr><td>Porušení vrstev</td><td>Omezeně</td><td>Ano</td></tr>
<tr><td>Natvrdo zapsané hodnoty napříč soubory</td><td>Omezeně</td><td>Ano</td></tr>
</tbody>
</table>

<p>Klíčovým poznatkem je, že tyto nástroje jsou <strong>komplementární</strong>, nikoli konkurenční. Linter zajišťuje, že každá cihla je dobře tvarovaná. Statická analýza zajišťuje, že budova stojí rovně.</p>

<h2 id="when-you-need-both">Kdy potřebujete obojí</h2>

<p>Každý profesionální projekt by měl mít linter. Otázkou je, kdy potřebujete také statickou analýzu na úrovni codebase. Zde jsou signály.</p>

<p><strong>Váš codebase má více než 100 souborů.</strong> Pod touto hranicí je většina architektonických problémů viditelná pouhým čtením. Nad ní se graf závislostí stává příliš složitým, aby ho bylo možné udržet v hlavě, a skryté cykly a mrtvý kód se začínají hromadit.</p>

<p><strong>Na projektu přispívá více vývojářů.</strong> Samostatný vývojář má dokonalý kontext o tom, který kód je aktivní a který mrtvý. Tým pěti vývojářů pracujících paralelně nevyhnutelně vytvoří mrtvý kód a náhodné cykly, protože žádná jednotlivá osoba nemá plnou viditelnost každé změny.</p>

<p><strong>Projekt je aktivní déle než šest měsíců.</strong> Technický dluh je funkcí času. Čím déle codebase žije, tím více mrtvého kódu, architektonického driftu a skryté provázanosti hromadí. Statická analýza poskytuje periodickou kontrolu zdraví, která zachytí tyto pomalu se pohybující problémy.</p>

<p><strong>Připravujete se na velký refaktoring.</strong> Před restrukturalizací codebase potřebujete porozumět jeho aktuálnímu tvaru. Které moduly jsou těsně provázané? Kde jsou cykly? Které soubory lze přesunout nezávisle? Statická analýza vám dá mapu, kterou potřebujete, než začnete provádět změny.</p>

<p><strong>AI agenti přispívají kódem.</strong> AI agenti pro kódování generují kód rychle, ale ne vždy mají plný kontext o architektonických normách vašeho projektu. Spouštění statické analýzy na příspěvcích generovaných AI zajišťuje, že nezavádějí nové cykly ani neporušují pravidla vrstev.</p>

<h2 id="how-aigiscode-complements-linters">Jak AigisCode doplňuje vaše lintery</h2>

<p>AigisCode je navržen tak, aby fungoval vedle vašeho stávajícího nastavení lintingu, nikoli aby jej nahrazoval. Ponecháte si ESLint pro styl JavaScriptu a typovou bezpečnost. Ponecháte si Pylint nebo Ruff pro konvence Pythonu. Přidáte AigisCode pro cross-file architektonickou analýzu, kterou lintery nedokáží poskytnout.</p>

<p>V praxi to znamená spouštět linter při každé změně souboru (v editoru i CI) a spouštět AigisCode periodicky nebo u pull requestů, které zasahují více než pár souborů. JSON report se čistě integruje do CI workflow a systém politik vám umožní vyladit citlivost detekce tak, aby odpovídala normám vašeho projektu.</p>

<p>Výsledkem jsou dvě komplementární vrstvy zajištění kvality kódu. Linter zachytí malé problémy okamžitě. Statický analyzátor zachytí strukturální problémy dříve, než se znásobí. Společně pokrývají celé spektrum kvality kódu, od jednotlivých příkazů po architektonickou integritu.</p>

<h2 id="the-right-tool-for-the-right-job">Správný nástroj pro správnou práci</h2>

<p>Debata linter versus statická analýza je falešná dichotomie. Je to jako ptát se, zda potřebujete mikroskop nebo teleskop. Dívají se na různé měřítka. Lintery se dívají na kód před vámi. Statická analýza se dívá na tvar celého vašeho codebase. V roce 2026, kdy codebase každé čtvrtletí rostou a stávají se složitějšími, potřebujete obojí. Otázka není „co z toho", ale „kdy přidám to druhé." Pro většinu týmů je odpověď: dříve, než si myslíte.</p>
`,
      fr: `
<p>If you have worked on a professional software project in the last decade, you have used a linter. ESLint for JavaScript, Pylint or Ruff for Python, PHPStan or Psalm for PHP, Clippy for Rust. Linters are ubiquitous, well-understood, and indispensable. They catch bugs, enforce style, and maintain consistency across teams. So why would anyone need something beyond a linter?</p>

<p>The answer lies in scope. Linters analyze files. Static analysis tools analyze codebases. The difference sounds subtle, but it changes everything about what problems you can detect.</p>

<h2 id="what-linters-do-well">What Linters Do Well</h2>

<p>Linters excel at <strong>intra-file analysis</strong>. Within a single file, a linter can detect unused variables, unreachable code branches, type mismatches, style violations, potential null dereferences, and dozens of other issues. Modern linters are remarkably sophisticated. TypeScript's built-in type checker performs deep flow analysis within functions. PHPStan at level 9 catches subtle type-narrowing issues that would be invisible to a human reviewer. Ruff can check 500 Python lint rules in under a second.</p>

<p>Linters also integrate seamlessly into the developer workflow. They run in your editor, providing real-time feedback as you type. They run in CI, blocking merges that introduce violations. They are fast, incremental, and deterministic. For file-level code quality, linters are the right tool.</p>

<h2 id="what-linters-miss">What Linters Miss</h2>

<p>The fundamental limitation of linters is that they process files independently. They do not build a graph of how your files relate to each other. This means they cannot detect an entire category of problems that only become visible when you look at the codebase as a connected system.</p>

<h3>Circular Dependencies</h3>

<p>A linter can see that <code>orders.py</code> imports from <code>inventory.py</code>. It can verify that the imported symbol exists and has the right type. What it cannot see is that <code>inventory.py</code> also imports from <code>orders.py</code>, creating a cycle. And it certainly cannot see that this cycle is part of a larger four-module loop that makes the entire order management subsystem impossible to test or deploy independently.</p>

<p>Detecting circular dependencies requires building a dependency graph of the entire codebase and running cycle-detection algorithms on it. This is fundamentally a codebase-level operation, not a file-level one.</p>

<h3>Cross-File Dead Code</h3>

<p>A linter can tell you that a variable is unused within a file. But it cannot tell you that an exported function is never imported by any other file in the project. It cannot tell you that an entire module has no inbound dependencies and is effectively orphaned. It cannot tell you that a class method is defined but never called from anywhere in the codebase.</p>

<p>Cross-file dead code detection requires knowing the complete import graph. Which modules import which symbols? Which class methods are called and from where? Which files are entry points and which are libraries? These questions cannot be answered by examining files in isolation.</p>

<h3>Architectural Violations</h3>

<p>Many teams define architectural rules, even if only informally. "Controllers should not import from other controllers." "The data layer should not depend on the presentation layer." "Utility modules should not import from feature modules." Linters cannot enforce these rules because they do not know about the layered structure of your codebase. They see individual files, not the dependency hierarchy between packages.</p>

<h3>God Classes and Bottleneck Files</h3>

<p>A god class, a class that has grown to handle too many responsibilities, is a well-known code smell. But measuring it requires more than counting lines of code. A true god class analysis looks at how many other modules depend on the class, how many different concerns it addresses, and whether its responsibilities can be separated without creating circular dependencies. Similarly, a bottleneck file, one that sits on too many dependency paths, can only be identified by analyzing the graph structure of the codebase.</p>

<h2 id="what-static-analysis-adds">What Static Analysis Adds</h2>

<p>Static analysis tools like AigisCode operate at the codebase level. They parse every file, extract symbols and dependencies, build a graph, and then run analysis algorithms on that graph. This enables an entirely different class of detections.</p>

<table>
<thead>
<tr><th>Capability</th><th>Linter</th><th>Static Analysis</th></tr>
</thead>
<tbody>
<tr><td>Unused variables in a file</td><td>Yes</td><td>Not the focus</td></tr>
<tr><td>Style enforcement</td><td>Yes</td><td>No</td></tr>
<tr><td>Type checking within functions</td><td>Yes</td><td>No</td></tr>
<tr><td>Circular dependencies</td><td>No</td><td>Yes</td></tr>
<tr><td>Cross-file dead code</td><td>No</td><td>Yes</td></tr>
<tr><td>Orphan files / modules</td><td>No</td><td>Yes</td></tr>
<tr><td>God classes (by coupling)</td><td>No</td><td>Yes</td></tr>
<tr><td>Bottleneck files</td><td>No</td><td>Yes</td></tr>
<tr><td>Layer violations</td><td>Limited</td><td>Yes</td></tr>
<tr><td>Hardwired values across files</td><td>Limited</td><td>Yes</td></tr>
</tbody>
</table>

<p>The key insight is that these tools are <strong>complementary</strong>, not competing. A linter ensures each brick is well-formed. Static analysis ensures the building stands straight.</p>

<h2 id="when-you-need-both">When You Need Both</h2>

<p>Every professional project should have a linter. The question is when you also need codebase-level static analysis. Here are the signals.</p>

<p><strong>Your codebase has more than 100 files.</strong> Below this threshold, most architectural issues are visible through casual reading. Above it, the dependency graph becomes too complex to hold in your head, and hidden cycles and dead code begin to accumulate.</p>

<p><strong>Multiple developers contribute to the project.</strong> A solo developer has perfect context about which code is active and which is dead. A team of five developers, working in parallel, will inevitably create dead code and accidental cycles because no single person has full visibility of every change.</p>

<p><strong>The project has been active for more than six months.</strong> Technical debt is a function of time. The longer a codebase lives, the more dead code, architectural drift, and hidden coupling it accumulates. Static analysis provides the periodic health check that catches these slow-moving issues.</p>

<p><strong>You are preparing for a major refactoring.</strong> Before restructuring a codebase, you need to understand its current shape. Which modules are tightly coupled? Where are the cycles? Which files can be moved independently? Static analysis gives you the map you need before you start making changes.</p>

<p><strong>AI agents are contributing code.</strong> AI coding agents generate code quickly but do not always have full context about the architectural norms of your project. Running static analysis on AI-generated contributions ensures they do not introduce new cycles or violate layering rules.</p>

<h2 id="how-aigiscode-complements-linters">How AigisCode Complements Your Linters</h2>

<p>AigisCode is designed to sit alongside your existing linting setup, not replace it. You keep ESLint for JavaScript style and type safety. You keep Pylint or Ruff for Python conventions. You add AigisCode for the cross-file, architectural analysis that linters cannot provide.</p>

<p>In practice, this means running your linter on every file change (in your editor and CI) and running AigisCode periodically or on pull requests that touch more than a handful of files. The JSON report integrates cleanly into CI workflows, and the policy system lets you tune detection sensitivity to match your project's norms.</p>

<p>The result is two complementary layers of code quality assurance. The linter catches the small issues immediately. The static analyzer catches the structural issues before they compound. Together, they cover the full spectrum of code quality, from individual statements to architectural integrity.</p>

<h2 id="the-right-tool-for-the-right-job">The Right Tool for the Right Job</h2>

<p>The linter versus static analysis debate is a false dichotomy. It is like asking whether you need a microscope or a telescope. They look at different scales. Linters look at the code in front of you. Static analysis looks at the shape of your entire codebase. In 2026, with codebases growing larger and more complex every quarter, you need both. The question is not "which one" but "when do I add the second one." For most teams, the answer is: sooner than you think.</p>
`,
      es: `
<p>If you have worked on a professional software project in the last decade, you have used a linter. ESLint for JavaScript, Pylint or Ruff for Python, PHPStan or Psalm for PHP, Clippy for Rust. Linters are ubiquitous, well-understood, and indispensable. They catch bugs, enforce style, and maintain consistency across teams. So why would anyone need something beyond a linter?</p>

<p>The answer lies in scope. Linters analyze files. Static analysis tools analyze codebases. The difference sounds subtle, but it changes everything about what problems you can detect.</p>

<h2 id="what-linters-do-well">What Linters Do Well</h2>

<p>Linters excel at <strong>intra-file analysis</strong>. Within a single file, a linter can detect unused variables, unreachable code branches, type mismatches, style violations, potential null dereferences, and dozens of other issues. Modern linters are remarkably sophisticated. TypeScript's built-in type checker performs deep flow analysis within functions. PHPStan at level 9 catches subtle type-narrowing issues that would be invisible to a human reviewer. Ruff can check 500 Python lint rules in under a second.</p>

<p>Linters also integrate seamlessly into the developer workflow. They run in your editor, providing real-time feedback as you type. They run in CI, blocking merges that introduce violations. They are fast, incremental, and deterministic. For file-level code quality, linters are the right tool.</p>

<h2 id="what-linters-miss">What Linters Miss</h2>

<p>The fundamental limitation of linters is that they process files independently. They do not build a graph of how your files relate to each other. This means they cannot detect an entire category of problems that only become visible when you look at the codebase as a connected system.</p>

<h3>Circular Dependencies</h3>

<p>A linter can see that <code>orders.py</code> imports from <code>inventory.py</code>. It can verify that the imported symbol exists and has the right type. What it cannot see is that <code>inventory.py</code> also imports from <code>orders.py</code>, creating a cycle. And it certainly cannot see that this cycle is part of a larger four-module loop that makes the entire order management subsystem impossible to test or deploy independently.</p>

<p>Detecting circular dependencies requires building a dependency graph of the entire codebase and running cycle-detection algorithms on it. This is fundamentally a codebase-level operation, not a file-level one.</p>

<h3>Cross-File Dead Code</h3>

<p>A linter can tell you that a variable is unused within a file. But it cannot tell you that an exported function is never imported by any other file in the project. It cannot tell you that an entire module has no inbound dependencies and is effectively orphaned. It cannot tell you that a class method is defined but never called from anywhere in the codebase.</p>

<p>Cross-file dead code detection requires knowing the complete import graph. Which modules import which symbols? Which class methods are called and from where? Which files are entry points and which are libraries? These questions cannot be answered by examining files in isolation.</p>

<h3>Architectural Violations</h3>

<p>Many teams define architectural rules, even if only informally. "Controllers should not import from other controllers." "The data layer should not depend on the presentation layer." "Utility modules should not import from feature modules." Linters cannot enforce these rules because they do not know about the layered structure of your codebase. They see individual files, not the dependency hierarchy between packages.</p>

<h3>God Classes and Bottleneck Files</h3>

<p>A god class, a class that has grown to handle too many responsibilities, is a well-known code smell. But measuring it requires more than counting lines of code. A true god class analysis looks at how many other modules depend on the class, how many different concerns it addresses, and whether its responsibilities can be separated without creating circular dependencies. Similarly, a bottleneck file, one that sits on too many dependency paths, can only be identified by analyzing the graph structure of the codebase.</p>

<h2 id="what-static-analysis-adds">What Static Analysis Adds</h2>

<p>Static analysis tools like AigisCode operate at the codebase level. They parse every file, extract symbols and dependencies, build a graph, and then run analysis algorithms on that graph. This enables an entirely different class of detections.</p>

<table>
<thead>
<tr><th>Capability</th><th>Linter</th><th>Static Analysis</th></tr>
</thead>
<tbody>
<tr><td>Unused variables in a file</td><td>Yes</td><td>Not the focus</td></tr>
<tr><td>Style enforcement</td><td>Yes</td><td>No</td></tr>
<tr><td>Type checking within functions</td><td>Yes</td><td>No</td></tr>
<tr><td>Circular dependencies</td><td>No</td><td>Yes</td></tr>
<tr><td>Cross-file dead code</td><td>No</td><td>Yes</td></tr>
<tr><td>Orphan files / modules</td><td>No</td><td>Yes</td></tr>
<tr><td>God classes (by coupling)</td><td>No</td><td>Yes</td></tr>
<tr><td>Bottleneck files</td><td>No</td><td>Yes</td></tr>
<tr><td>Layer violations</td><td>Limited</td><td>Yes</td></tr>
<tr><td>Hardwired values across files</td><td>Limited</td><td>Yes</td></tr>
</tbody>
</table>

<p>The key insight is that these tools are <strong>complementary</strong>, not competing. A linter ensures each brick is well-formed. Static analysis ensures the building stands straight.</p>

<h2 id="when-you-need-both">When You Need Both</h2>

<p>Every professional project should have a linter. The question is when you also need codebase-level static analysis. Here are the signals.</p>

<p><strong>Your codebase has more than 100 files.</strong> Below this threshold, most architectural issues are visible through casual reading. Above it, the dependency graph becomes too complex to hold in your head, and hidden cycles and dead code begin to accumulate.</p>

<p><strong>Multiple developers contribute to the project.</strong> A solo developer has perfect context about which code is active and which is dead. A team of five developers, working in parallel, will inevitably create dead code and accidental cycles because no single person has full visibility of every change.</p>

<p><strong>The project has been active for more than six months.</strong> Technical debt is a function of time. The longer a codebase lives, the more dead code, architectural drift, and hidden coupling it accumulates. Static analysis provides the periodic health check that catches these slow-moving issues.</p>

<p><strong>You are preparing for a major refactoring.</strong> Before restructuring a codebase, you need to understand its current shape. Which modules are tightly coupled? Where are the cycles? Which files can be moved independently? Static analysis gives you the map you need before you start making changes.</p>

<p><strong>AI agents are contributing code.</strong> AI coding agents generate code quickly but do not always have full context about the architectural norms of your project. Running static analysis on AI-generated contributions ensures they do not introduce new cycles or violate layering rules.</p>

<h2 id="how-aigiscode-complements-linters">How AigisCode Complements Your Linters</h2>

<p>AigisCode is designed to sit alongside your existing linting setup, not replace it. You keep ESLint for JavaScript style and type safety. You keep Pylint or Ruff for Python conventions. You add AigisCode for the cross-file, architectural analysis that linters cannot provide.</p>

<p>In practice, this means running your linter on every file change (in your editor and CI) and running AigisCode periodically or on pull requests that touch more than a handful of files. The JSON report integrates cleanly into CI workflows, and the policy system lets you tune detection sensitivity to match your project's norms.</p>

<p>The result is two complementary layers of code quality assurance. The linter catches the small issues immediately. The static analyzer catches the structural issues before they compound. Together, they cover the full spectrum of code quality, from individual statements to architectural integrity.</p>

<h2 id="the-right-tool-for-the-right-job">The Right Tool for the Right Job</h2>

<p>The linter versus static analysis debate is a false dichotomy. It is like asking whether you need a microscope or a telescope. They look at different scales. Linters look at the code in front of you. Static analysis looks at the shape of your entire codebase. In 2026, with codebases growing larger and more complex every quarter, you need both. The question is not "which one" but "when do I add the second one." For most teams, the answer is: sooner than you think.</p>
`,
      zh: `
<p>If you have worked on a professional software project in the last decade, you have used a linter. ESLint for JavaScript, Pylint or Ruff for Python, PHPStan or Psalm for PHP, Clippy for Rust. Linters are ubiquitous, well-understood, and indispensable. They catch bugs, enforce style, and maintain consistency across teams. So why would anyone need something beyond a linter?</p>

<p>The answer lies in scope. Linters analyze files. Static analysis tools analyze codebases. The difference sounds subtle, but it changes everything about what problems you can detect.</p>

<h2 id="what-linters-do-well">What Linters Do Well</h2>

<p>Linters excel at <strong>intra-file analysis</strong>. Within a single file, a linter can detect unused variables, unreachable code branches, type mismatches, style violations, potential null dereferences, and dozens of other issues. Modern linters are remarkably sophisticated. TypeScript's built-in type checker performs deep flow analysis within functions. PHPStan at level 9 catches subtle type-narrowing issues that would be invisible to a human reviewer. Ruff can check 500 Python lint rules in under a second.</p>

<p>Linters also integrate seamlessly into the developer workflow. They run in your editor, providing real-time feedback as you type. They run in CI, blocking merges that introduce violations. They are fast, incremental, and deterministic. For file-level code quality, linters are the right tool.</p>

<h2 id="what-linters-miss">What Linters Miss</h2>

<p>The fundamental limitation of linters is that they process files independently. They do not build a graph of how your files relate to each other. This means they cannot detect an entire category of problems that only become visible when you look at the codebase as a connected system.</p>

<h3>Circular Dependencies</h3>

<p>A linter can see that <code>orders.py</code> imports from <code>inventory.py</code>. It can verify that the imported symbol exists and has the right type. What it cannot see is that <code>inventory.py</code> also imports from <code>orders.py</code>, creating a cycle. And it certainly cannot see that this cycle is part of a larger four-module loop that makes the entire order management subsystem impossible to test or deploy independently.</p>

<p>Detecting circular dependencies requires building a dependency graph of the entire codebase and running cycle-detection algorithms on it. This is fundamentally a codebase-level operation, not a file-level one.</p>

<h3>Cross-File Dead Code</h3>

<p>A linter can tell you that a variable is unused within a file. But it cannot tell you that an exported function is never imported by any other file in the project. It cannot tell you that an entire module has no inbound dependencies and is effectively orphaned. It cannot tell you that a class method is defined but never called from anywhere in the codebase.</p>

<p>Cross-file dead code detection requires knowing the complete import graph. Which modules import which symbols? Which class methods are called and from where? Which files are entry points and which are libraries? These questions cannot be answered by examining files in isolation.</p>

<h3>Architectural Violations</h3>

<p>Many teams define architectural rules, even if only informally. "Controllers should not import from other controllers." "The data layer should not depend on the presentation layer." "Utility modules should not import from feature modules." Linters cannot enforce these rules because they do not know about the layered structure of your codebase. They see individual files, not the dependency hierarchy between packages.</p>

<h3>God Classes and Bottleneck Files</h3>

<p>A god class, a class that has grown to handle too many responsibilities, is a well-known code smell. But measuring it requires more than counting lines of code. A true god class analysis looks at how many other modules depend on the class, how many different concerns it addresses, and whether its responsibilities can be separated without creating circular dependencies. Similarly, a bottleneck file, one that sits on too many dependency paths, can only be identified by analyzing the graph structure of the codebase.</p>

<h2 id="what-static-analysis-adds">What Static Analysis Adds</h2>

<p>Static analysis tools like AigisCode operate at the codebase level. They parse every file, extract symbols and dependencies, build a graph, and then run analysis algorithms on that graph. This enables an entirely different class of detections.</p>

<table>
<thead>
<tr><th>Capability</th><th>Linter</th><th>Static Analysis</th></tr>
</thead>
<tbody>
<tr><td>Unused variables in a file</td><td>Yes</td><td>Not the focus</td></tr>
<tr><td>Style enforcement</td><td>Yes</td><td>No</td></tr>
<tr><td>Type checking within functions</td><td>Yes</td><td>No</td></tr>
<tr><td>Circular dependencies</td><td>No</td><td>Yes</td></tr>
<tr><td>Cross-file dead code</td><td>No</td><td>Yes</td></tr>
<tr><td>Orphan files / modules</td><td>No</td><td>Yes</td></tr>
<tr><td>God classes (by coupling)</td><td>No</td><td>Yes</td></tr>
<tr><td>Bottleneck files</td><td>No</td><td>Yes</td></tr>
<tr><td>Layer violations</td><td>Limited</td><td>Yes</td></tr>
<tr><td>Hardwired values across files</td><td>Limited</td><td>Yes</td></tr>
</tbody>
</table>

<p>The key insight is that these tools are <strong>complementary</strong>, not competing. A linter ensures each brick is well-formed. Static analysis ensures the building stands straight.</p>

<h2 id="when-you-need-both">When You Need Both</h2>

<p>Every professional project should have a linter. The question is when you also need codebase-level static analysis. Here are the signals.</p>

<p><strong>Your codebase has more than 100 files.</strong> Below this threshold, most architectural issues are visible through casual reading. Above it, the dependency graph becomes too complex to hold in your head, and hidden cycles and dead code begin to accumulate.</p>

<p><strong>Multiple developers contribute to the project.</strong> A solo developer has perfect context about which code is active and which is dead. A team of five developers, working in parallel, will inevitably create dead code and accidental cycles because no single person has full visibility of every change.</p>

<p><strong>The project has been active for more than six months.</strong> Technical debt is a function of time. The longer a codebase lives, the more dead code, architectural drift, and hidden coupling it accumulates. Static analysis provides the periodic health check that catches these slow-moving issues.</p>

<p><strong>You are preparing for a major refactoring.</strong> Before restructuring a codebase, you need to understand its current shape. Which modules are tightly coupled? Where are the cycles? Which files can be moved independently? Static analysis gives you the map you need before you start making changes.</p>

<p><strong>AI agents are contributing code.</strong> AI coding agents generate code quickly but do not always have full context about the architectural norms of your project. Running static analysis on AI-generated contributions ensures they do not introduce new cycles or violate layering rules.</p>

<h2 id="how-aigiscode-complements-linters">How AigisCode Complements Your Linters</h2>

<p>AigisCode is designed to sit alongside your existing linting setup, not replace it. You keep ESLint for JavaScript style and type safety. You keep Pylint or Ruff for Python conventions. You add AigisCode for the cross-file, architectural analysis that linters cannot provide.</p>

<p>In practice, this means running your linter on every file change (in your editor and CI) and running AigisCode periodically or on pull requests that touch more than a handful of files. The JSON report integrates cleanly into CI workflows, and the policy system lets you tune detection sensitivity to match your project's norms.</p>

<p>The result is two complementary layers of code quality assurance. The linter catches the small issues immediately. The static analyzer catches the structural issues before they compound. Together, they cover the full spectrum of code quality, from individual statements to architectural integrity.</p>

<h2 id="the-right-tool-for-the-right-job">The Right Tool for the Right Job</h2>

<p>The linter versus static analysis debate is a false dichotomy. It is like asking whether you need a microscope or a telescope. They look at different scales. Linters look at the code in front of you. Static analysis looks at the shape of your entire codebase. In 2026, with codebases growing larger and more complex every quarter, you need both. The question is not "which one" but "when do I add the second one." For most teams, the answer is: sooner than you think.</p>
`,
      hi: `
<p>If you have worked on a professional software project in the last decade, you have used a linter. ESLint for JavaScript, Pylint or Ruff for Python, PHPStan or Psalm for PHP, Clippy for Rust. Linters are ubiquitous, well-understood, and indispensable. They catch bugs, enforce style, and maintain consistency across teams. So why would anyone need something beyond a linter?</p>

<p>The answer lies in scope. Linters analyze files. Static analysis tools analyze codebases. The difference sounds subtle, but it changes everything about what problems you can detect.</p>

<h2 id="what-linters-do-well">What Linters Do Well</h2>

<p>Linters excel at <strong>intra-file analysis</strong>. Within a single file, a linter can detect unused variables, unreachable code branches, type mismatches, style violations, potential null dereferences, and dozens of other issues. Modern linters are remarkably sophisticated. TypeScript's built-in type checker performs deep flow analysis within functions. PHPStan at level 9 catches subtle type-narrowing issues that would be invisible to a human reviewer. Ruff can check 500 Python lint rules in under a second.</p>

<p>Linters also integrate seamlessly into the developer workflow. They run in your editor, providing real-time feedback as you type. They run in CI, blocking merges that introduce violations. They are fast, incremental, and deterministic. For file-level code quality, linters are the right tool.</p>

<h2 id="what-linters-miss">What Linters Miss</h2>

<p>The fundamental limitation of linters is that they process files independently. They do not build a graph of how your files relate to each other. This means they cannot detect an entire category of problems that only become visible when you look at the codebase as a connected system.</p>

<h3>Circular Dependencies</h3>

<p>A linter can see that <code>orders.py</code> imports from <code>inventory.py</code>. It can verify that the imported symbol exists and has the right type. What it cannot see is that <code>inventory.py</code> also imports from <code>orders.py</code>, creating a cycle. And it certainly cannot see that this cycle is part of a larger four-module loop that makes the entire order management subsystem impossible to test or deploy independently.</p>

<p>Detecting circular dependencies requires building a dependency graph of the entire codebase and running cycle-detection algorithms on it. This is fundamentally a codebase-level operation, not a file-level one.</p>

<h3>Cross-File Dead Code</h3>

<p>A linter can tell you that a variable is unused within a file. But it cannot tell you that an exported function is never imported by any other file in the project. It cannot tell you that an entire module has no inbound dependencies and is effectively orphaned. It cannot tell you that a class method is defined but never called from anywhere in the codebase.</p>

<p>Cross-file dead code detection requires knowing the complete import graph. Which modules import which symbols? Which class methods are called and from where? Which files are entry points and which are libraries? These questions cannot be answered by examining files in isolation.</p>

<h3>Architectural Violations</h3>

<p>Many teams define architectural rules, even if only informally. "Controllers should not import from other controllers." "The data layer should not depend on the presentation layer." "Utility modules should not import from feature modules." Linters cannot enforce these rules because they do not know about the layered structure of your codebase. They see individual files, not the dependency hierarchy between packages.</p>

<h3>God Classes and Bottleneck Files</h3>

<p>A god class, a class that has grown to handle too many responsibilities, is a well-known code smell. But measuring it requires more than counting lines of code. A true god class analysis looks at how many other modules depend on the class, how many different concerns it addresses, and whether its responsibilities can be separated without creating circular dependencies. Similarly, a bottleneck file, one that sits on too many dependency paths, can only be identified by analyzing the graph structure of the codebase.</p>

<h2 id="what-static-analysis-adds">What Static Analysis Adds</h2>

<p>Static analysis tools like AigisCode operate at the codebase level. They parse every file, extract symbols and dependencies, build a graph, and then run analysis algorithms on that graph. This enables an entirely different class of detections.</p>

<table>
<thead>
<tr><th>Capability</th><th>Linter</th><th>Static Analysis</th></tr>
</thead>
<tbody>
<tr><td>Unused variables in a file</td><td>Yes</td><td>Not the focus</td></tr>
<tr><td>Style enforcement</td><td>Yes</td><td>No</td></tr>
<tr><td>Type checking within functions</td><td>Yes</td><td>No</td></tr>
<tr><td>Circular dependencies</td><td>No</td><td>Yes</td></tr>
<tr><td>Cross-file dead code</td><td>No</td><td>Yes</td></tr>
<tr><td>Orphan files / modules</td><td>No</td><td>Yes</td></tr>
<tr><td>God classes (by coupling)</td><td>No</td><td>Yes</td></tr>
<tr><td>Bottleneck files</td><td>No</td><td>Yes</td></tr>
<tr><td>Layer violations</td><td>Limited</td><td>Yes</td></tr>
<tr><td>Hardwired values across files</td><td>Limited</td><td>Yes</td></tr>
</tbody>
</table>

<p>The key insight is that these tools are <strong>complementary</strong>, not competing. A linter ensures each brick is well-formed. Static analysis ensures the building stands straight.</p>

<h2 id="when-you-need-both">When You Need Both</h2>

<p>Every professional project should have a linter. The question is when you also need codebase-level static analysis. Here are the signals.</p>

<p><strong>Your codebase has more than 100 files.</strong> Below this threshold, most architectural issues are visible through casual reading. Above it, the dependency graph becomes too complex to hold in your head, and hidden cycles and dead code begin to accumulate.</p>

<p><strong>Multiple developers contribute to the project.</strong> A solo developer has perfect context about which code is active and which is dead. A team of five developers, working in parallel, will inevitably create dead code and accidental cycles because no single person has full visibility of every change.</p>

<p><strong>The project has been active for more than six months.</strong> Technical debt is a function of time. The longer a codebase lives, the more dead code, architectural drift, and hidden coupling it accumulates. Static analysis provides the periodic health check that catches these slow-moving issues.</p>

<p><strong>You are preparing for a major refactoring.</strong> Before restructuring a codebase, you need to understand its current shape. Which modules are tightly coupled? Where are the cycles? Which files can be moved independently? Static analysis gives you the map you need before you start making changes.</p>

<p><strong>AI agents are contributing code.</strong> AI coding agents generate code quickly but do not always have full context about the architectural norms of your project. Running static analysis on AI-generated contributions ensures they do not introduce new cycles or violate layering rules.</p>

<h2 id="how-aigiscode-complements-linters">How AigisCode Complements Your Linters</h2>

<p>AigisCode is designed to sit alongside your existing linting setup, not replace it. You keep ESLint for JavaScript style and type safety. You keep Pylint or Ruff for Python conventions. You add AigisCode for the cross-file, architectural analysis that linters cannot provide.</p>

<p>In practice, this means running your linter on every file change (in your editor and CI) and running AigisCode periodically or on pull requests that touch more than a handful of files. The JSON report integrates cleanly into CI workflows, and the policy system lets you tune detection sensitivity to match your project's norms.</p>

<p>The result is two complementary layers of code quality assurance. The linter catches the small issues immediately. The static analyzer catches the structural issues before they compound. Together, they cover the full spectrum of code quality, from individual statements to architectural integrity.</p>

<h2 id="the-right-tool-for-the-right-job">The Right Tool for the Right Job</h2>

<p>The linter versus static analysis debate is a false dichotomy. It is like asking whether you need a microscope or a telescope. They look at different scales. Linters look at the code in front of you. Static analysis looks at the shape of your entire codebase. In 2026, with codebases growing larger and more complex every quarter, you need both. The question is not "which one" but "when do I add the second one." For most teams, the answer is: sooner than you think.</p>
`,
      pt: `
<p>If you have worked on a professional software project in the last decade, you have used a linter. ESLint for JavaScript, Pylint or Ruff for Python, PHPStan or Psalm for PHP, Clippy for Rust. Linters are ubiquitous, well-understood, and indispensable. They catch bugs, enforce style, and maintain consistency across teams. So why would anyone need something beyond a linter?</p>

<p>The answer lies in scope. Linters analyze files. Static analysis tools analyze codebases. The difference sounds subtle, but it changes everything about what problems you can detect.</p>

<h2 id="what-linters-do-well">What Linters Do Well</h2>

<p>Linters excel at <strong>intra-file analysis</strong>. Within a single file, a linter can detect unused variables, unreachable code branches, type mismatches, style violations, potential null dereferences, and dozens of other issues. Modern linters are remarkably sophisticated. TypeScript's built-in type checker performs deep flow analysis within functions. PHPStan at level 9 catches subtle type-narrowing issues that would be invisible to a human reviewer. Ruff can check 500 Python lint rules in under a second.</p>

<p>Linters also integrate seamlessly into the developer workflow. They run in your editor, providing real-time feedback as you type. They run in CI, blocking merges that introduce violations. They are fast, incremental, and deterministic. For file-level code quality, linters are the right tool.</p>

<h2 id="what-linters-miss">What Linters Miss</h2>

<p>The fundamental limitation of linters is that they process files independently. They do not build a graph of how your files relate to each other. This means they cannot detect an entire category of problems that only become visible when you look at the codebase as a connected system.</p>

<h3>Circular Dependencies</h3>

<p>A linter can see that <code>orders.py</code> imports from <code>inventory.py</code>. It can verify that the imported symbol exists and has the right type. What it cannot see is that <code>inventory.py</code> also imports from <code>orders.py</code>, creating a cycle. And it certainly cannot see that this cycle is part of a larger four-module loop that makes the entire order management subsystem impossible to test or deploy independently.</p>

<p>Detecting circular dependencies requires building a dependency graph of the entire codebase and running cycle-detection algorithms on it. This is fundamentally a codebase-level operation, not a file-level one.</p>

<h3>Cross-File Dead Code</h3>

<p>A linter can tell you that a variable is unused within a file. But it cannot tell you that an exported function is never imported by any other file in the project. It cannot tell you that an entire module has no inbound dependencies and is effectively orphaned. It cannot tell you that a class method is defined but never called from anywhere in the codebase.</p>

<p>Cross-file dead code detection requires knowing the complete import graph. Which modules import which symbols? Which class methods are called and from where? Which files are entry points and which are libraries? These questions cannot be answered by examining files in isolation.</p>

<h3>Architectural Violations</h3>

<p>Many teams define architectural rules, even if only informally. "Controllers should not import from other controllers." "The data layer should not depend on the presentation layer." "Utility modules should not import from feature modules." Linters cannot enforce these rules because they do not know about the layered structure of your codebase. They see individual files, not the dependency hierarchy between packages.</p>

<h3>God Classes and Bottleneck Files</h3>

<p>A god class, a class that has grown to handle too many responsibilities, is a well-known code smell. But measuring it requires more than counting lines of code. A true god class analysis looks at how many other modules depend on the class, how many different concerns it addresses, and whether its responsibilities can be separated without creating circular dependencies. Similarly, a bottleneck file, one that sits on too many dependency paths, can only be identified by analyzing the graph structure of the codebase.</p>

<h2 id="what-static-analysis-adds">What Static Analysis Adds</h2>

<p>Static analysis tools like AigisCode operate at the codebase level. They parse every file, extract symbols and dependencies, build a graph, and then run analysis algorithms on that graph. This enables an entirely different class of detections.</p>

<table>
<thead>
<tr><th>Capability</th><th>Linter</th><th>Static Analysis</th></tr>
</thead>
<tbody>
<tr><td>Unused variables in a file</td><td>Yes</td><td>Not the focus</td></tr>
<tr><td>Style enforcement</td><td>Yes</td><td>No</td></tr>
<tr><td>Type checking within functions</td><td>Yes</td><td>No</td></tr>
<tr><td>Circular dependencies</td><td>No</td><td>Yes</td></tr>
<tr><td>Cross-file dead code</td><td>No</td><td>Yes</td></tr>
<tr><td>Orphan files / modules</td><td>No</td><td>Yes</td></tr>
<tr><td>God classes (by coupling)</td><td>No</td><td>Yes</td></tr>
<tr><td>Bottleneck files</td><td>No</td><td>Yes</td></tr>
<tr><td>Layer violations</td><td>Limited</td><td>Yes</td></tr>
<tr><td>Hardwired values across files</td><td>Limited</td><td>Yes</td></tr>
</tbody>
</table>

<p>The key insight is that these tools are <strong>complementary</strong>, not competing. A linter ensures each brick is well-formed. Static analysis ensures the building stands straight.</p>

<h2 id="when-you-need-both">When You Need Both</h2>

<p>Every professional project should have a linter. The question is when you also need codebase-level static analysis. Here are the signals.</p>

<p><strong>Your codebase has more than 100 files.</strong> Below this threshold, most architectural issues are visible through casual reading. Above it, the dependency graph becomes too complex to hold in your head, and hidden cycles and dead code begin to accumulate.</p>

<p><strong>Multiple developers contribute to the project.</strong> A solo developer has perfect context about which code is active and which is dead. A team of five developers, working in parallel, will inevitably create dead code and accidental cycles because no single person has full visibility of every change.</p>

<p><strong>The project has been active for more than six months.</strong> Technical debt is a function of time. The longer a codebase lives, the more dead code, architectural drift, and hidden coupling it accumulates. Static analysis provides the periodic health check that catches these slow-moving issues.</p>

<p><strong>You are preparing for a major refactoring.</strong> Before restructuring a codebase, you need to understand its current shape. Which modules are tightly coupled? Where are the cycles? Which files can be moved independently? Static analysis gives you the map you need before you start making changes.</p>

<p><strong>AI agents are contributing code.</strong> AI coding agents generate code quickly but do not always have full context about the architectural norms of your project. Running static analysis on AI-generated contributions ensures they do not introduce new cycles or violate layering rules.</p>

<h2 id="how-aigiscode-complements-linters">How AigisCode Complements Your Linters</h2>

<p>AigisCode is designed to sit alongside your existing linting setup, not replace it. You keep ESLint for JavaScript style and type safety. You keep Pylint or Ruff for Python conventions. You add AigisCode for the cross-file, architectural analysis that linters cannot provide.</p>

<p>In practice, this means running your linter on every file change (in your editor and CI) and running AigisCode periodically or on pull requests that touch more than a handful of files. The JSON report integrates cleanly into CI workflows, and the policy system lets you tune detection sensitivity to match your project's norms.</p>

<p>The result is two complementary layers of code quality assurance. The linter catches the small issues immediately. The static analyzer catches the structural issues before they compound. Together, they cover the full spectrum of code quality, from individual statements to architectural integrity.</p>

<h2 id="the-right-tool-for-the-right-job">The Right Tool for the Right Job</h2>

<p>The linter versus static analysis debate is a false dichotomy. It is like asking whether you need a microscope or a telescope. They look at different scales. Linters look at the code in front of you. Static analysis looks at the shape of your entire codebase. In 2026, with codebases growing larger and more complex every quarter, you need both. The question is not "which one" but "when do I add the second one." For most teams, the answer is: sooner than you think.</p>
`,
      ar: `
<p>If you have worked on a professional software project in the last decade, you have used a linter. ESLint for JavaScript, Pylint or Ruff for Python, PHPStan or Psalm for PHP, Clippy for Rust. Linters are ubiquitous, well-understood, and indispensable. They catch bugs, enforce style, and maintain consistency across teams. So why would anyone need something beyond a linter?</p>

<p>The answer lies in scope. Linters analyze files. Static analysis tools analyze codebases. The difference sounds subtle, but it changes everything about what problems you can detect.</p>

<h2 id="what-linters-do-well">What Linters Do Well</h2>

<p>Linters excel at <strong>intra-file analysis</strong>. Within a single file, a linter can detect unused variables, unreachable code branches, type mismatches, style violations, potential null dereferences, and dozens of other issues. Modern linters are remarkably sophisticated. TypeScript's built-in type checker performs deep flow analysis within functions. PHPStan at level 9 catches subtle type-narrowing issues that would be invisible to a human reviewer. Ruff can check 500 Python lint rules in under a second.</p>

<p>Linters also integrate seamlessly into the developer workflow. They run in your editor, providing real-time feedback as you type. They run in CI, blocking merges that introduce violations. They are fast, incremental, and deterministic. For file-level code quality, linters are the right tool.</p>

<h2 id="what-linters-miss">What Linters Miss</h2>

<p>The fundamental limitation of linters is that they process files independently. They do not build a graph of how your files relate to each other. This means they cannot detect an entire category of problems that only become visible when you look at the codebase as a connected system.</p>

<h3>Circular Dependencies</h3>

<p>A linter can see that <code>orders.py</code> imports from <code>inventory.py</code>. It can verify that the imported symbol exists and has the right type. What it cannot see is that <code>inventory.py</code> also imports from <code>orders.py</code>, creating a cycle. And it certainly cannot see that this cycle is part of a larger four-module loop that makes the entire order management subsystem impossible to test or deploy independently.</p>

<p>Detecting circular dependencies requires building a dependency graph of the entire codebase and running cycle-detection algorithms on it. This is fundamentally a codebase-level operation, not a file-level one.</p>

<h3>Cross-File Dead Code</h3>

<p>A linter can tell you that a variable is unused within a file. But it cannot tell you that an exported function is never imported by any other file in the project. It cannot tell you that an entire module has no inbound dependencies and is effectively orphaned. It cannot tell you that a class method is defined but never called from anywhere in the codebase.</p>

<p>Cross-file dead code detection requires knowing the complete import graph. Which modules import which symbols? Which class methods are called and from where? Which files are entry points and which are libraries? These questions cannot be answered by examining files in isolation.</p>

<h3>Architectural Violations</h3>

<p>Many teams define architectural rules, even if only informally. "Controllers should not import from other controllers." "The data layer should not depend on the presentation layer." "Utility modules should not import from feature modules." Linters cannot enforce these rules because they do not know about the layered structure of your codebase. They see individual files, not the dependency hierarchy between packages.</p>

<h3>God Classes and Bottleneck Files</h3>

<p>A god class, a class that has grown to handle too many responsibilities, is a well-known code smell. But measuring it requires more than counting lines of code. A true god class analysis looks at how many other modules depend on the class, how many different concerns it addresses, and whether its responsibilities can be separated without creating circular dependencies. Similarly, a bottleneck file, one that sits on too many dependency paths, can only be identified by analyzing the graph structure of the codebase.</p>

<h2 id="what-static-analysis-adds">What Static Analysis Adds</h2>

<p>Static analysis tools like AigisCode operate at the codebase level. They parse every file, extract symbols and dependencies, build a graph, and then run analysis algorithms on that graph. This enables an entirely different class of detections.</p>

<table>
<thead>
<tr><th>Capability</th><th>Linter</th><th>Static Analysis</th></tr>
</thead>
<tbody>
<tr><td>Unused variables in a file</td><td>Yes</td><td>Not the focus</td></tr>
<tr><td>Style enforcement</td><td>Yes</td><td>No</td></tr>
<tr><td>Type checking within functions</td><td>Yes</td><td>No</td></tr>
<tr><td>Circular dependencies</td><td>No</td><td>Yes</td></tr>
<tr><td>Cross-file dead code</td><td>No</td><td>Yes</td></tr>
<tr><td>Orphan files / modules</td><td>No</td><td>Yes</td></tr>
<tr><td>God classes (by coupling)</td><td>No</td><td>Yes</td></tr>
<tr><td>Bottleneck files</td><td>No</td><td>Yes</td></tr>
<tr><td>Layer violations</td><td>Limited</td><td>Yes</td></tr>
<tr><td>Hardwired values across files</td><td>Limited</td><td>Yes</td></tr>
</tbody>
</table>

<p>The key insight is that these tools are <strong>complementary</strong>, not competing. A linter ensures each brick is well-formed. Static analysis ensures the building stands straight.</p>

<h2 id="when-you-need-both">When You Need Both</h2>

<p>Every professional project should have a linter. The question is when you also need codebase-level static analysis. Here are the signals.</p>

<p><strong>Your codebase has more than 100 files.</strong> Below this threshold, most architectural issues are visible through casual reading. Above it, the dependency graph becomes too complex to hold in your head, and hidden cycles and dead code begin to accumulate.</p>

<p><strong>Multiple developers contribute to the project.</strong> A solo developer has perfect context about which code is active and which is dead. A team of five developers, working in parallel, will inevitably create dead code and accidental cycles because no single person has full visibility of every change.</p>

<p><strong>The project has been active for more than six months.</strong> Technical debt is a function of time. The longer a codebase lives, the more dead code, architectural drift, and hidden coupling it accumulates. Static analysis provides the periodic health check that catches these slow-moving issues.</p>

<p><strong>You are preparing for a major refactoring.</strong> Before restructuring a codebase, you need to understand its current shape. Which modules are tightly coupled? Where are the cycles? Which files can be moved independently? Static analysis gives you the map you need before you start making changes.</p>

<p><strong>AI agents are contributing code.</strong> AI coding agents generate code quickly but do not always have full context about the architectural norms of your project. Running static analysis on AI-generated contributions ensures they do not introduce new cycles or violate layering rules.</p>

<h2 id="how-aigiscode-complements-linters">How AigisCode Complements Your Linters</h2>

<p>AigisCode is designed to sit alongside your existing linting setup, not replace it. You keep ESLint for JavaScript style and type safety. You keep Pylint or Ruff for Python conventions. You add AigisCode for the cross-file, architectural analysis that linters cannot provide.</p>

<p>In practice, this means running your linter on every file change (in your editor and CI) and running AigisCode periodically or on pull requests that touch more than a handful of files. The JSON report integrates cleanly into CI workflows, and the policy system lets you tune detection sensitivity to match your project's norms.</p>

<p>The result is two complementary layers of code quality assurance. The linter catches the small issues immediately. The static analyzer catches the structural issues before they compound. Together, they cover the full spectrum of code quality, from individual statements to architectural integrity.</p>

<h2 id="the-right-tool-for-the-right-job">The Right Tool for the Right Job</h2>

<p>The linter versus static analysis debate is a false dichotomy. It is like asking whether you need a microscope or a telescope. They look at different scales. Linters look at the code in front of you. Static analysis looks at the shape of your entire codebase. In 2026, with codebases growing larger and more complex every quarter, you need both. The question is not "which one" but "when do I add the second one." For most teams, the answer is: sooner than you think.</p>
`,
      pl: `<h2 id="static-vs-linters">Analiza statyczna vs lintery</h2>
<p>ESLint i Pylint sprawdzają pliki pojedynczo. AigisCode analizuje cały graf zależności. Te podejścia są komplementarne.</p>`,
      bn: `
<p>গত দশকে যদি আপনি কোনো পেশাদার সফটওয়্যার প্রজেক্টে কাজ করে থাকেন, তাহলে আপনি একটি লিন্টার ব্যবহার করেছেন। JavaScript-এর জন্য ESLint, Python-এর জন্য Pylint বা Ruff, PHP-র জন্য PHPStan বা Psalm, Rust-এর জন্য Clippy। লিন্টারগুলো সর্বব্যাপী, সুবোধ্য এবং অপরিহার্য। তারা বাগ ধরে, স্টাইল প্রয়োগ করে এবং টিম জুড়ে সামঞ্জস্য বজায় রাখে। তাহলে কেন কারো লিন্টারের বাইরে কিছু দরকার হবে?</p>

<p>উত্তর স্কোপে নিহিত। লিন্টারগুলো ফাইল বিশ্লেষণ করে। স্ট্যাটিক অ্যানালিসিস টুলগুলো কোডবেস বিশ্লেষণ করে। পার্থক্যটি সূক্ষ্ম মনে হয়, কিন্তু এটি আপনি কোন সমস্যাগুলো শনাক্ত করতে পারেন সে সম্পর্কে সবকিছু পরিবর্তন করে।</p>

<h2 id="what-linters-do-well">লিন্টারগুলো কী ভালো করে</h2>

<p>লিন্টারগুলো <strong>ইন্ট্রা-ফাইল বিশ্লেষণে</strong> পারদর্শী। একটি একক ফাইলের মধ্যে, একটি লিন্টার অব্যবহৃত ভেরিয়েবল, অপ্রাপ্য কোড ব্রাঞ্চ, টাইপ মিসম্যাচ, স্টাইল লঙ্ঘন, সম্ভাব্য null dereference, এবং আরো ডজনখানেক সমস্যা শনাক্ত করতে পারে। আধুনিক লিন্টারগুলো অসাধারণভাবে পরিশীলিত। TypeScript-এর বিল্ট-ইন টাইপ চেকার ফাংশনের মধ্যে গভীর ফ্লো বিশ্লেষণ করে। লেভেল ৯-এ PHPStan সূক্ষ্ম টাইপ-ন্যারোইং সমস্যা ধরে যা মানুষের রিভিউয়ারের কাছে অদৃশ্য হতো। Ruff এক সেকেন্ডেরও কম সময়ে ৫০০টি Python lint নিয়ম পরীক্ষা করতে পারে।</p>

<p>লিন্টারগুলো ডেভেলপার ওয়ার্কফ্লোতে নির্বিঘ্নে ইন্টিগ্রেট হয়। তারা আপনার এডিটরে চলে, আপনি টাইপ করার সাথে সাথে রিয়েল-টাইম ফিডব্যাক দেয়। তারা CI-তে চলে, লঙ্ঘন প্রবর্তনকারী মার্জ ব্লক করে। তারা দ্রুত, ইনক্রিমেন্টাল এবং ডিটারমিনিস্টিক। ফাইল-লেভেল কোড কোয়ালিটির জন্য, লিন্টারগুলোই সঠিক টুল।</p>

<h2 id="what-linters-miss">লিন্টারগুলো কী মিস করে</h2>

<p>লিন্টারগুলোর মৌলিক সীমাবদ্ধতা হলো তারা ফাইলগুলো স্বাধীনভাবে প্রসেস করে। তারা আপনার ফাইলগুলো কিভাবে একে অপরের সাথে সম্পর্কিত তার একটি গ্রাফ তৈরি করে না। এর মানে তারা সমস্যার একটি সম্পূর্ণ বিভাগ শনাক্ত করতে পারে না যা শুধুমাত্র তখনই দৃশ্যমান হয় যখন আপনি কোডবেসকে একটি সংযুক্ত সিস্টেম হিসেবে দেখেন।</p>

<h3>সার্কুলার ডিপেন্ডেন্সি</h3>

<p>একটি লিন্টার দেখতে পারে যে <code>orders.py</code> <code>inventory.py</code> থেকে ইমপোর্ট করে। এটি যাচাই করতে পারে যে ইমপোর্ট করা সিম্বল বিদ্যমান এবং সঠিক টাইপ আছে। যা এটি দেখতে পারে না তা হলো <code>inventory.py</code>ও <code>orders.py</code> থেকে ইমপোর্ট করে, একটি সাইকেল তৈরি করে। এবং এটি অবশ্যই দেখতে পারে না যে এই সাইকেল একটি বড় চার-মডিউল লুপের অংশ যা সম্পূর্ণ অর্ডার ম্যানেজমেন্ট সাবসিস্টেমকে স্বাধীনভাবে টেস্ট বা ডিপ্লয় করা অসম্ভব করে তোলে।</p>

<p>সার্কুলার ডিপেন্ডেন্সি শনাক্ত করতে সম্পূর্ণ কোডবেসের একটি ডিপেন্ডেন্সি গ্রাফ তৈরি করা এবং এতে সাইকেল-শনাক্তকরণ অ্যালগরিদম চালানো প্রয়োজন। এটি মৌলিকভাবে একটি কোডবেস-স্তরের অপারেশন, ফাইল-স্তরের নয়।</p>

<h3>ক্রস-ফাইল ডেড কোড</h3>

<p>একটি লিন্টার আপনাকে বলতে পারে যে একটি ফাইলের মধ্যে একটি ভেরিয়েবল অব্যবহৃত। কিন্তু এটি আপনাকে বলতে পারে না যে একটি এক্সপোর্ট করা ফাংশন প্রজেক্টের অন্য কোনো ফাইল দ্বারা কখনো ইমপোর্ট করা হয় না। এটি আপনাকে বলতে পারে না যে একটি সম্পূর্ণ মডিউলের কোনো ইনবাউন্ড ডিপেন্ডেন্সি নেই এবং কার্যত অনাথ। এটি আপনাকে বলতে পারে না যে একটি ক্লাস মেথড সংজ্ঞায়িত কিন্তু কোডবেসের কোথাও থেকে কখনো কল করা হয় না।</p>

<p>ক্রস-ফাইল ডেড কোড শনাক্তকরণের জন্য সম্পূর্ণ ইমপোর্ট গ্রাফ জানা প্রয়োজন। কোন মডিউল কোন সিম্বল ইমপোর্ট করে? কোন ক্লাস মেথড কোথা থেকে কল হয়? কোন ফাইলগুলো এন্ট্রি পয়েন্ট এবং কোনগুলো লাইব্রেরি? এই প্রশ্নগুলো ফাইল আলাদাভাবে পরীক্ষা করে উত্তর দেওয়া যায় না।</p>

<h3>আর্কিটেকচারাল লঙ্ঘন</h3>

<p>অনেক টিম আর্কিটেকচারাল নিয়ম সংজ্ঞায়িত করে, যদিও শুধু অনানুষ্ঠানিকভাবে। "কন্ট্রোলার অন্য কন্ট্রোলার থেকে ইমপোর্ট করা উচিত নয়।" "ডেটা লেয়ার প্রেজেন্টেশন লেয়ারের উপর নির্ভর করা উচিত নয়।" "ইউটিলিটি মডিউল ফিচার মডিউল থেকে ইমপোর্ট করা উচিত নয়।" লিন্টারগুলো এই নিয়ম প্রয়োগ করতে পারে না কারণ তারা আপনার কোডবেসের লেয়ার্ড স্ট্রাকচার সম্পর্কে জানে না। তারা পৃথক ফাইল দেখে, প্যাকেজগুলোর মধ্যে ডিপেন্ডেন্সি হায়ারার্কি নয়।</p>

<h3>God Class এবং বটলনেক ফাইল</h3>

<p>একটি god class, একটি ক্লাস যা অনেক বেশি দায়িত্ব সামলাতে বেড়ে গেছে, একটি সুপরিচিত কোড স্মেল। কিন্তু এটি পরিমাপ করতে কোড লাইন গণনার চেয়ে বেশি প্রয়োজন। একটি প্রকৃত god class বিশ্লেষণ দেখে কতগুলো অন্য মডিউল ক্লাসটির উপর নির্ভর করে, এটি কতগুলো ভিন্ন উদ্বেগ সমাধান করে এবং সার্কুলার ডিপেন্ডেন্সি তৈরি না করে এর দায়িত্বগুলো আলাদা করা যায় কিনা। একইভাবে, একটি বটলনেক ফাইল, যা অনেক বেশি ডিপেন্ডেন্সি পাথে বসে, শুধুমাত্র কোডবেসের গ্রাফ স্ট্রাকচার বিশ্লেষণ করে চিহ্নিত করা যায়।</p>

<h2 id="what-static-analysis-adds">স্ট্যাটিক অ্যানালিসিস কী যোগ করে</h2>

<p>AigisCode-এর মতো স্ট্যাটিক অ্যানালিসিস টুলগুলো কোডবেস স্তরে কাজ করে। তারা প্রতিটি ফাইল পার্স করে, সিম্বল এবং ডিপেন্ডেন্সি এক্সট্র্যাক্ট করে, একটি গ্রাফ তৈরি করে, এবং তারপর সেই গ্রাফে বিশ্লেষণ অ্যালগরিদম চালায়। এটি সম্পূর্ণ ভিন্ন শ্রেণীর শনাক্তকরণ সক্ষম করে।</p>

<table>
<thead>
<tr><th>ক্ষমতা</th><th>লিন্টার</th><th>স্ট্যাটিক অ্যানালিসিস</th></tr>
</thead>
<tbody>
<tr><td>ফাইলে অব্যবহৃত ভেরিয়েবল</td><td>হ্যাঁ</td><td>ফোকাস নয়</td></tr>
<tr><td>স্টাইল প্রয়োগ</td><td>হ্যাঁ</td><td>না</td></tr>
<tr><td>ফাংশনের মধ্যে টাইপ চেকিং</td><td>হ্যাঁ</td><td>না</td></tr>
<tr><td>সার্কুলার ডিপেন্ডেন্সি</td><td>না</td><td>হ্যাঁ</td></tr>
<tr><td>ক্রস-ফাইল ডেড কোড</td><td>না</td><td>হ্যাঁ</td></tr>
<tr><td>অনাথ ফাইল / মডিউল</td><td>না</td><td>হ্যাঁ</td></tr>
<tr><td>God class (কাপলিং দ্বারা)</td><td>না</td><td>হ্যাঁ</td></tr>
<tr><td>বটলনেক ফাইল</td><td>না</td><td>হ্যাঁ</td></tr>
<tr><td>লেয়ার লঙ্ঘন</td><td>সীমিত</td><td>হ্যাঁ</td></tr>
<tr><td>ফাইল জুড়ে হার্ডওয়্যার্ড ভ্যালু</td><td>সীমিত</td><td>হ্যাঁ</td></tr>
</tbody>
</table>

<p>মূল অন্তর্দৃষ্টি হলো এই টুলগুলো <strong>পরিপূরক</strong>, প্রতিযোগী নয়। একটি লিন্টার নিশ্চিত করে প্রতিটি ইট সুগঠিত। স্ট্যাটিক অ্যানালিসিস নিশ্চিত করে বিল্ডিং সোজা দাঁড়িয়ে আছে।</p>

<h2 id="when-you-need-both">আপনার কখন দুটোই দরকার</h2>

<p>প্রতিটি পেশাদার প্রজেক্টের একটি লিন্টার থাকা উচিত। প্রশ্ন হলো কখন আপনার কোডবেস-স্তরের স্ট্যাটিক অ্যানালিসিসও দরকার। এখানে সংকেতগুলো।</p>

<p><strong>আপনার কোডবেসে ১০০টির বেশি ফাইল আছে।</strong> এই থ্রেশহোল্ডের নীচে, বেশিরভাগ আর্কিটেকচারাল সমস্যা সাধারণ পড়ার মাধ্যমে দৃশ্যমান। এর উপরে, ডিপেন্ডেন্সি গ্রাফ আপনার মাথায় ধরে রাখতে খুব জটিল হয়ে যায়, এবং লুকানো সাইকেল এবং ডেড কোড জমা হতে শুরু করে।</p>

<p><strong>একাধিক ডেভেলপার প্রজেক্টে অবদান রাখে।</strong> একজন একা ডেভেলপারের কোন কোড সক্রিয় এবং কোনটি ডেড সে সম্পর্কে নিখুঁত প্রসঙ্গ থাকে। পাঁচজন ডেভেলপারের একটি টিম, সমান্তরালে কাজ করছে, অনিবার্যভাবে ডেড কোড এবং দুর্ঘটনাজনিত সাইকেল তৈরি করবে কারণ কোনো একক ব্যক্তির প্রতিটি পরিবর্তনের সম্পূর্ণ দৃশ্যমানতা নেই।</p>

<p><strong>প্রজেক্ট ছয় মাসের বেশি সময় ধরে সক্রিয়।</strong> টেকনিক্যাল ডেট সময়ের একটি ফাংশন। একটি কোডবেস যত বেশি দিন বেঁচে থাকে, তত বেশি ডেড কোড, আর্কিটেকচারাল ড্রিফ্ট এবং লুকানো কাপলিং জমা হয়। স্ট্যাটিক অ্যানালিসিস পর্যায়ক্রমিক স্বাস্থ্য পরীক্ষা প্রদান করে যা এই ধীর-গতির সমস্যাগুলো ধরে।</p>

<p><strong>আপনি একটি বড় রিফ্যাক্টরিংয়ের জন্য প্রস্তুতি নিচ্ছেন।</strong> একটি কোডবেস পুনর্গঠন করার আগে, আপনাকে এর বর্তমান আকার বুঝতে হবে। কোন মডিউলগুলো শক্তভাবে কাপল্ড? সাইকেলগুলো কোথায়? কোন ফাইলগুলো স্বাধীনভাবে সরানো যায়? আপনি পরিবর্তন করা শুরু করার আগে স্ট্যাটিক অ্যানালিসিস আপনাকে প্রয়োজনীয় মানচিত্র দেয়।</p>

<p><strong>AI এজেন্ট কোড অবদান রাখছে।</strong> AI কোডিং এজেন্ট দ্রুত কোড তৈরি করে কিন্তু সবসময় আপনার প্রজেক্টের আর্কিটেকচারাল নর্ম সম্পর্কে সম্পূর্ণ প্রসঙ্গ থাকে না। AI-উৎপন্ন অবদানে স্ট্যাটিক অ্যানালিসিস চালানো নিশ্চিত করে তারা নতুন সাইকেল প্রবর্তন করে না বা লেয়ারিং নিয়ম লঙ্ঘন করে না।</p>

<h2 id="how-aigiscode-complements-linters">AigisCode কিভাবে আপনার লিন্টারের পরিপূরক</h2>

<p>AigisCode আপনার বিদ্যমান লিন্টিং সেটআপের পাশে বসার জন্য ডিজাইন করা হয়েছে, প্রতিস্থাপন করার জন্য নয়। আপনি JavaScript স্টাইল এবং টাইপ সেফটির জন্য ESLint রাখুন। আপনি Python কনভেনশনের জন্য Pylint বা Ruff রাখুন। আপনি ক্রস-ফাইল, আর্কিটেকচারাল বিশ্লেষণের জন্য AigisCode যোগ করুন যা লিন্টারগুলো প্রদান করতে পারে না।</p>

<p>বাস্তবে, এর মানে প্রতিটি ফাইল পরিবর্তনে আপনার লিন্টার চালানো (আপনার এডিটর এবং CI-তে) এবং পর্যায়ক্রমে বা পুল রিকোয়েস্টে AigisCode চালানো যা কয়েকটির বেশি ফাইল স্পর্শ করে। JSON রিপোর্ট CI ওয়ার্কফ্লোতে পরিষ্কারভাবে ইন্টিগ্রেট হয়, এবং পলিসি সিস্টেম আপনাকে আপনার প্রজেক্টের নর্মের সাথে মেলাতে শনাক্তকরণ সংবেদনশীলতা টিউন করতে দেয়।</p>

<p>ফলাফল হলো কোড কোয়ালিটি নিশ্চয়তার দুটি পরিপূরক স্তর। লিন্টার অবিলম্বে ছোট সমস্যাগুলো ধরে। স্ট্যাটিক অ্যানালাইজার স্ট্রাকচারাল সমস্যাগুলো সংযুক্ত হওয়ার আগেই ধরে। একসাথে, তারা কোড কোয়ালিটির সম্পূর্ণ বর্ণালী কভার করে, পৃথক স্টেটমেন্ট থেকে আর্কিটেকচারাল ইন্টিগ্রিটি পর্যন্ত।</p>

<h2 id="the-right-tool-for-the-right-job">সঠিক কাজের জন্য সঠিক টুল</h2>

<p>লিন্টার বনাম স্ট্যাটিক অ্যানালিসিস বিতর্ক একটি মিথ্যা দ্বিধা। এটি জিজ্ঞাসা করার মতো আপনার মাইক্রোস্কোপ দরকার নাকি টেলিস্কোপ। তারা ভিন্ন স্কেলে দেখে। লিন্টারগুলো আপনার সামনের কোড দেখে। স্ট্যাটিক অ্যানালিসিস আপনার সম্পূর্ণ কোডবেসের আকার দেখে। ২০২৬ সালে, প্রতি ত্রৈমাসিকে কোডবেসগুলো বড় এবং জটিল হচ্ছে, আপনার দুটোই দরকার। প্রশ্ন "কোনটি" নয় বরং "কখন আমি দ্বিতীয়টি যোগ করব।" বেশিরভাগ টিমের জন্য, উত্তর হলো: আপনি যা ভাবেন তার চেয়ে তাড়াতাড়ি।</p>
`,
    },
  },

  /* ======================================================================== */
  /*  5. How AI Coding Agents Use AigisCode                                   */
  /* ======================================================================== */
  {
    slug: 'ai-agents-code-quality-workflow',
    date: '2026-03-03',
    readTime: 9,
    tags: ['AI Agents', 'Automation', 'Code Review', 'Workflow'],
    image: '/blog-ai-agents-workflow.jpg',
    author: { name: 'David Strejc', role: 'Creator of AigisCode' },
    relatedSlugs: [
      'why-ai-code-analysis-matters-2026',
      'static-analysis-vs-linters-2026',
    ],
    title: {
      en: 'How AI Coding Agents Use AigisCode for Automated Code Quality',
      cs: 'Jak AI agenti používají AigisCode pro automatizovanou kvalitu kódu',
      fr: 'Comment les agents IA utilisent AigisCode pour la qualite de code automatisee',
      es: 'Como los agentes de IA usan AigisCode para la calidad de codigo automatizada',
      zh: 'AI 编程代理如何使用 AigisCode 实现自动化代码质量',
      hi: 'AI कोडिंग एजेंट AigisCode का उपयोग स्वचालित कोड गुणवत्ता के लिए कैसे करते हैं',
      pt: 'Como agentes de IA usam o AigisCode para qualidade de código automatizada',
      ar: 'كيف يستخدم وكلاء البرمجة بالذكاء الاصطناعي AigisCode لجودة الشيفرة الآلية',
      pl: 'Jak agenci AI do kodowania używają AigisCode do automatycznej jakości kodu',
      bn: 'AI কোডিং এজেন্টরা কিভাবে স্বয়ংক্রিয় কোড কোয়ালিটির জন্য AigisCode ব্যবহার করে',
    },
    description: {
      en: 'AI coding agents need structured, machine-readable codebase health data. Learn how agents like Claude and Codex consume AigisCode reports to autonomously triage and fix architectural issues.',
      cs: 'AI agenti potřebují strukturovaná data o zdraví kódu. Zjistěte, jak agenti konzumují reporty AigisCode.',
      fr: 'Les agents IA ont besoin de donnees structurees sur la sante du codebase. Decouvrez comment ils utilisent les rapports AigisCode.',
      es: 'Los agentes de IA necesitan datos estructurados sobre la salud del codigo. Descubra como consumen los reportes de AigisCode.',
      zh: 'AI 编程代理需要结构化的代码库健康数据。了解代理如何消费 AigisCode 报告。',
      hi: 'AI एजेंट्स को संरचित कोडबेस स्वास्थ्य डेटा की आवश्यकता होती है। जानें कि एजेंट AigisCode रिपोर्ट का उपयोग कैसे करते हैं।',
      pt: 'Agentes de IA precisam de dados estruturados sobre a saúde do código. Saiba como eles consomem relatórios do AigisCode.',
      ar: 'يحتاج وكلاء البرمجة بالذكاء الاصطناعي إلى بيانات صحة قاعدة الشيفرة المنظمة والقابلة للقراءة آلياً. تعرّف كيف يستهلك وكلاء مثل Claude وCodex تقارير AigisCode لفرز ومعالجة المشكلات المعمارية تلقائياً.',
      pl: 'Agenci AI do kodowania potrzebują ustrukturyzowanych, czytelnych maszynowo danych o kondycji bazy kodu. Dowiedz się, jak agenci tacy jak Claude i Codex używają wyjścia JSON AigisCode do autonomicznego rozumienia, naprawiania i poprawy jakości kodu.',
      bn: 'AI কোডিং এজেন্টদের কাঠামোবদ্ধ, মেশিন-রিডেবল কোডবেস স্বাস্থ্য ডেটা দরকার। Claude এবং Codex-এর মতো এজেন্টরা কিভাবে স্বায়ত্তশাসিতভাবে আর্কিটেকচারাল সমস্যা ট্রায়াজ ও সমাধান করতে AigisCode রিপোর্ট ব্যবহার করে তা জানুন।',
    },
    metaDescription: {
      en: 'Learn how AI coding agents like Claude Code and Codex use AigisCode for automated code quality. Explore the analyze-parse-triage-fix workflow, policy integration, and the future of AI-driven maintenance.',
      cs: 'Zjistěte, jak AI agenti jako Claude Code používají AigisCode pro automatizovanou kvalitu kódu.',
      fr: 'Decouvrez comment les agents IA comme Claude Code utilisent AigisCode pour la qualite automatisee du code.',
      es: 'Descubra como los agentes IA como Claude Code usan AigisCode para la calidad automatizada del codigo.',
      zh: '了解 Claude Code 等 AI 代理如何使用 AigisCode 实现自动化代码质量。',
      hi: 'जानें कि Claude Code जैसे AI एजेंट AigisCode का उपयोग स्वचालित कोड गुणवत्ता के लिए कैसे करते हैं।',
      pt: 'Saiba como agentes de IA como Claude Code usam o AigisCode para qualidade de código automatizada.',
      ar: 'تعرّف كيف يستخدم وكلاء البرمجة بالذكاء الاصطناعي مثل Claude Code وCodex أداة AigisCode لجودة الشيفرة الآلية. استكشف سير عمل التحليل-التحليل-الفرز-الإصلاح وتكامل السياسات ومستقبل الصيانة المدفوعة بالذكاء الاصطناعي.',
      pl: 'Dowiedz się, jak agenci AI tacy jak Claude Code i Codex używają AigisCode do automatycznej jakości kodu. Poznaj workflow agenta, strukturę raportów JSON i wzorce integracji.',
      bn: 'জানুন Claude Code এবং Codex-এর মতো AI কোডিং এজেন্টরা কিভাবে স্বয়ংক্রিয় কোড কোয়ালিটির জন্য AigisCode ব্যবহার করে। analyze-parse-triage-fix ওয়ার্কফ্লো, পলিসি ইন্টিগ্রেশন এবং AI-চালিত রক্ষণাবেক্ষণের ভবিষ্যৎ অন্বেষণ করুন।',
    },
    content: {
      en: `
<p>The development workflow of 2026 looks nothing like what we had five years ago. AI coding agents, autonomous systems that can read, understand, modify, and test code, are now a regular part of software engineering. Agents like Claude Code, GitHub Copilot Workspace, and Codex can implement features from issue descriptions, fix bugs from error logs, and refactor code based on architectural goals. But every agent shares a common need: structured, reliable information about the codebase they are working in.</p>

<p>This is where AigisCode fits into the AI agent workflow. Not as another AI that generates code, but as the analytical layer that gives agents the context they need to make good decisions about code quality and architecture.</p>

<h2 id="the-agent-workflow">The Analyze-Parse-Triage-Fix Workflow</h2>

<p>The recommended workflow for an AI agent using AigisCode follows four stages, each building on the previous one.</p>

<h3>Stage 1: Analyze</h3>

<p>The agent runs <code>aigiscode analyze /path/to/project</code>. This executes the full six-stage pipeline: indexing source files, building the dependency graph, running detectors, applying saved exclusion rules, running AI review for classification, and generating the report. The output is a structured JSON file at <code>.aigiscode/deterministic-analysis.json</code>.</p>

<p>This step is deterministic except for the optional AI review stage. An agent can run with <code>--skip-ai</code> for fully deterministic analysis, which is useful in CI environments where reproducibility matters more than nuanced classification.</p>

<h3>Stage 2: Parse</h3>

<p>The agent reads the JSON report and extracts structured findings. The report is organized into clear sections that an agent can navigate programmatically.</p>

<p><code>graph_analysis.strong_circular_dependencies</code> contains the architectural cycles that need structural refactoring. Each cycle lists the participating modules and the specific import edges that form the loop. <code>graph_analysis.god_classes</code> identifies classes with excessive coupling and responsibility. <code>graph_analysis.bottleneck_files</code> highlights files that sit on too many dependency paths. <code>dead_code</code> catalogs unused imports, unreferenced methods, abandoned classes, and orphan files. <code>hardwiring</code> lists magic strings, repeated literals, and hardcoded network addresses.</p>

<p>Each finding includes a confidence level (high, medium, low) and a severity rating, giving the agent enough information to make triage decisions without human input.</p>

<h3>Stage 3: Triage</h3>

<p>This is where the agent applies judgment. Not all findings are equally important, and not all findings are true positives. A well-designed agent triage workflow looks like this.</p>

<p>First, filter by confidence. Start with high-confidence findings. These have the lowest false-positive rate and provide the most reliable signal about actual issues. Medium-confidence findings should be sampled and verified before acting on them in bulk. Low-confidence findings are informational.</p>

<p>Second, prioritize by impact. A circular dependency between two core modules that every feature imports is more impactful than a circular dependency between two utility scripts. A dead code finding in a module with 50 inbound dependencies matters more than one in an isolated test helper.</p>

<p>Third, classify the fix type. Some findings require simple deletions (unused imports, orphan files). Some require interface changes (circular dependencies). Some require configuration updates (hardwired values). The agent should batch similar fixes together for efficient execution.</p>

<h3>Stage 4: Fix</h3>

<p>The agent applies fixes based on the triage results. For simple fixes like removing unused imports or deleting orphan files, the agent can act with high confidence. For structural fixes like breaking circular dependencies, the agent should propose a plan, implement it, and run the test suite to verify that the change does not break anything.</p>

<p>After applying fixes, the agent runs <code>aigiscode analyze /path/to/project</code> to regenerate the report from the existing index. This is faster than a full analysis because it skips the indexing stage. The agent can then compare the new report to the baseline to verify that findings have been resolved and no new issues have been introduced.</p>

<h2 id="how-agents-consume-reports">How Real Agents Consume AigisCode Reports</h2>

<p>Different AI agents have different strengths, and the way they consume AigisCode reports reflects those differences.</p>

<h3>Claude Code</h3>

<p>Claude Code agents excel at understanding context and making nuanced decisions. When given an AigisCode report, a Claude agent can read the full findings, understand the relationships between them, and develop a coherent refactoring plan that addresses multiple issues simultaneously. For example, if the report shows a circular dependency between the <code>auth</code> and <code>users</code> modules and also dead code in both modules, Claude can propose a single refactoring that breaks the cycle and removes the dead code in one coherent change.</p>

<p>The structured JSON format is particularly well-suited for Claude's context window. The agent can load the entire report, cross-reference findings with the actual source code, and produce fixes that account for the full context of each issue.</p>

<h3>Codex Agents</h3>

<p>Codex agents are effective at executing targeted fixes within a defined scope. They work well when given a specific finding from the AigisCode report and asked to fix it. For example, given a dead code finding for a specific unused method, a Codex agent can identify all related code, verify the method is truly unused, and produce a clean deletion with updated imports.</p>

<p>The AigisCode workers module (<code>workers/codex.py</code>) provides integration points specifically designed for Codex-style agents that process findings one at a time.</p>

<h2 id="policy-driven-behavior">Policy-Driven Agent Behavior</h2>

<p>One of AigisCode's key design principles is that project-specific behavior should live in policy, not in code. This principle extends naturally to AI agent workflows.</p>

<p>When an agent encounters a false positive, it should not modify the analyzer. Instead, it should add an exclusion rule to <code>.aigiscode/rules.json</code>. When it identifies a pattern of false positives (for example, all methods in a <code>Contracts/</code> directory are flagged as dead code because they are implemented via interfaces), it should encode the pattern in <code>.aigiscode/policy.json</code>.</p>

<p>This approach has several advantages. First, it keeps the agent's changes reviewable. A policy change is a small JSON modification that a human reviewer can evaluate quickly. Second, it keeps the analysis reproducible. Other agents and other developers running AigisCode on the same project will benefit from the accumulated policy knowledge. Third, it follows the principle of least privilege. An agent updating policy files cannot accidentally break the analysis tool itself.</p>

<p>The policy file supports a rich set of configuration options. Import aliases (<code>js_import_aliases</code>) tell the graph builder how to resolve path aliases like <code>@/</code> in TypeScript projects. Entry point patterns (<code>orphan_entry_patterns</code>) mark files that are legitimate entry points even though nothing imports them, like CLI scripts or test fixtures. Abandonment patterns (<code>abandoned_entry_patterns</code>) tell the dead code detector which directories contain interface implementations that should not be flagged.</p>

<h2 id="practical-integration-examples">Practical Integration Examples</h2>

<h3>CI Pipeline Integration</h3>

<p>The most common integration pattern is running AigisCode in CI and having an agent process the results. The CI pipeline runs <code>aigiscode analyze .</code> on every pull request. If new findings appear compared to the baseline on the main branch, the pipeline triggers an agent to review the findings, classify them, and either fix them automatically or leave review comments on the PR.</p>

<pre><code># .github/workflows/code-quality.yml
- name: Run AigisCode Analysis
  run: aigiscode analyze .
- name: Compare with baseline
  run: diff .aigiscode/deterministic-analysis.json baseline-report.json
- name: Agent review (if new findings)
  run: agent review --report .aigiscode/deterministic-analysis.json</code></pre>

<h3>Scheduled Maintenance</h3>

<p>Another pattern is scheduled codebase maintenance. An agent runs AigisCode weekly, reviews the full report, and creates a maintenance PR that addresses the highest-priority findings. This creates a steady cadence of structural improvement without requiring developers to manually triage architectural issues.</p>

<h3>Onboarding Reports</h3>

<p>When a new developer or a new AI agent begins working on a codebase, running AigisCode provides an instant structural overview. The report shows where the dependency hotspots are, which modules are most coupled, and where the known technical debt lives. This is faster and more reliable than reading documentation that may be out of date.</p>

<h2 id="the-future-of-ai-driven-maintenance">The Future of AI-Driven Maintenance</h2>

<p>We are moving toward a world where codebase maintenance is largely automated. AI agents will continuously monitor code quality metrics, detect degradation, and apply fixes without human intervention for routine issues. Human developers will focus on architectural decisions, product design, and the high-level guidance that agents need to work effectively.</p>

<p>AigisCode is designed for this future. Its structured output, policy-driven behavior, and clear separation between deterministic analysis and AI classification make it a natural fit for agent-driven workflows. The tool provides the eyes. The agent provides the hands. And the policy file provides the institutional knowledge that ensures both work together effectively.</p>

<p>The teams that integrate these tools into their workflow now will have a significant advantage. Not just cleaner code, but a system that gets smarter over time as the policy accumulates project-specific knowledge and the agents become better at interpreting and acting on the analysis results. The future of code quality is not a better linter. It is an intelligent system where analysis, policy, and autonomous agents work together to keep codebases healthy at scale.</p>
`,
      cs: `
<p>Vývojový workflow roku 2026 vypadá zcela jinak než to, co jsme měli před pěti lety. AI agenti pro kódování — autonomní systémy, které dokáží číst, chápat, upravovat a testovat kód — jsou nyní běžnou součástí softwarového inženýrství. Agenti jako Claude Code, GitHub Copilot Workspace a Codex dokáží implementovat funkce z popisů issues, opravovat chyby z error logů a refaktorovat kód na základě architektonických cílů. Ale každý agent sdílí společnou potřebu: strukturované, spolehlivé informace o codebase, ve kterém pracuje.</p>

<p>Zde se AigisCode začleňuje do workflow AI agentů. Ne jako další AI, která generuje kód, ale jako analytická vrstva, která dává agentům kontext potřebný k dobrým rozhodnutím o kvalitě kódu a architektuře.</p>

<h2 id="the-agent-workflow">Workflow Analyze-Parse-Triage-Fix</h2>

<p>Doporučený workflow pro AI agenta používajícího AigisCode sleduje čtyři fáze, z nichž každá navazuje na předchozí.</p>

<h3>Fáze 1: Analýza</h3>

<p>Agent spustí <code>aigiscode analyze /path/to/project</code>. To spustí kompletní šestistupňovou pipeline: indexaci zdrojových souborů, budování grafu závislostí, spouštění detektorů, aplikaci uložených pravidel výjimek, spuštění AI review pro klasifikaci a generování reportu. Výstupem je strukturovaný JSON soubor v <code>.aigiscode/deterministic-analysis.json</code>.</p>

<p>Tento krok je deterministický s výjimkou volitelné fáze AI review. Agent může spustit s <code>--skip-ai</code> pro plně deterministickou analýzu, což je užitečné v CI prostředích, kde reprodukovatelnost záleží více než nuancovaná klasifikace.</p>

<h3>Fáze 2: Parsování</h3>

<p>Agent čte JSON report a extrahuje strukturované nálezy. Report je organizován do jasných sekcí, kterými může agent navigovat programaticky.</p>

<p><code>graph_analysis.strong_circular_dependencies</code> obsahuje architektonické cykly, které potřebují strukturální refaktoring. Každý cyklus uvádí zúčastněné moduly a specifické importní hrany tvořící smyčku. <code>graph_analysis.god_classes</code> identifikuje třídy s nadměrnou provázaností a odpovědností. <code>graph_analysis.bottleneck_files</code> zvýrazňuje soubory na příliš mnoha cestách závislostí. <code>dead_code</code> katalogizuje nepoužívané importy, neodkazované metody, opuštěné třídy a osiřelé soubory. <code>hardwiring</code> vypisuje magické řetězce, opakované literály a natvrdo zapsané síťové adresy.</p>

<p>Každý nález zahrnuje úroveň spolehlivosti (vysoká, střední, nízká) a hodnocení závažnosti, čímž dává agentovi dostatek informací pro rozhodování o třídění bez lidského vstupu.</p>

<h3>Fáze 3: Třídění</h3>

<p>Zde agent uplatňuje úsudek. Ne všechny nálezy jsou stejně důležité a ne všechny nálezy jsou true positives. Dobře navržený workflow třídění agenta vypadá takto.</p>

<p>Zaprvé, filtrování podle spolehlivosti. Začněte s nálezy s vysokou spolehlivostí. Ty mají nejnižší míru false positives a poskytují nejspolehlivější signál o skutečných problémech. Nálezy se střední spolehlivostí by měly být vzorkovány a ověřeny před hromadným jednáním. Nálezy s nízkou spolehlivostí jsou informativní.</p>

<p>Zadruhé, prioritizace podle dopadu. Cyklická závislost mezi dvěma jádrovými moduly, které importuje každá funkce, má větší dopad než cyklická závislost mezi dvěma utilitními skripty. Nález mrtvého kódu v modulu s 50 příchozími závislostmi záleží více než jeden v izolovaném testovacím helperu.</p>

<p>Zatřetí, klasifikace typu opravy. Některé nálezy vyžadují jednoduché smazání (nepoužívané importy, osiřelé soubory). Některé vyžadují změny rozhraní (cyklické závislosti). Některé vyžadují aktualizace konfigurace (natvrdo zapsané hodnoty). Agent by měl podobné opravy seskupovat pro efektivní provedení.</p>

<h3>Fáze 4: Oprava</h3>

<p>Agent aplikuje opravy na základě výsledků třídění. Pro jednoduché opravy jako odstranění nepoužívaných importů nebo smazání osiřelých souborů může agent jednat s vysokou spolehlivostí. Pro strukturální opravy jako rozbití cyklických závislostí by měl agent navrhnout plán, implementovat jej a spustit testovací sadu pro ověření, že změna nic nerozbije.</p>

<p>Po aplikaci oprav agent spustí <code>aigiscode analyze /path/to/project</code> pro regeneraci reportu z existujícího indexu. To je rychlejší než plná analýza, protože přeskakuje fázi indexace. Agent pak může porovnat nový report se základní linií a ověřit, že nálezy byly vyřešeny a nebyly zavedeny žádné nové problémy.</p>

<h2 id="how-agents-consume-reports">Jak skuteční agenti konzumují reporty AigisCode</h2>

<p>Různí AI agenti mají různé silné stránky a způsob, jakým konzumují reporty AigisCode, tyto rozdíly odráží.</p>

<h3>Claude Code</h3>

<p>Agenti Claude Code vynikají v porozumění kontextu a nuancovaném rozhodování. Když dostanou report AigisCode, agent Claude dokáže přečíst kompletní nálezy, pochopit vztahy mezi nimi a vyvinout koherentní plán refaktoringu, který řeší více problémů současně. Například pokud report ukazuje cyklickou závislost mezi moduly <code>auth</code> a <code>users</code> a současně mrtvý kód v obou modulech, Claude může navrhnout jediný refaktoring, který přeruší cyklus a odstraní mrtvý kód v jedné koherentní změně.</p>

<p>Strukturovaný JSON formát je obzvláště vhodný pro kontextové okno Claude. Agent může načíst celý report, křížově odkazovat nálezy se skutečným zdrojovým kódem a produkovat opravy, které zohledňují plný kontext každého problému.</p>

<h3>Agenti Codex</h3>

<p>Agenti Codex jsou efektivní při provádění cílených oprav v definovaném rozsahu. Fungují dobře, když dostanou specifický nález z reportu AigisCode a jsou požádáni o jeho opravu. Například při nálezu mrtvého kódu pro specifickou nepoužívanou metodu dokáže agent Codex identifikovat veškerý související kód, ověřit, že metoda je skutečně nepoužívaná, a vytvořit čisté smazání s aktualizovanými importy.</p>

<p>Modul workers AigisCode (<code>workers/codex.py</code>) poskytuje integrační body speciálně navržené pro agenty stylu Codex, kteří zpracovávají nálezy jeden po druhém.</p>

<h2 id="policy-driven-behavior">Chování agentů řízené politikami</h2>

<p>Jedním z klíčových designových principů AigisCode je, že chování specifické pro projekt by mělo žít v politice, nikoli v kódu. Tento princip se přirozeně rozšiřuje na workflow AI agentů.</p>

<p>Když agent narazí na false positive, neměl by modifikovat analyzátor. Místo toho by měl přidat pravidlo výjimky do <code>.aigiscode/rules.json</code>. Když identifikuje vzorec false positives (například všechny metody v adresáři <code>Contracts/</code> jsou označeny jako mrtvý kód, protože jsou implementovány přes rozhraní), měl by vzorec zakódovat do <code>.aigiscode/policy.json</code>.</p>

<p>Tento přístup má několik výhod. Zaprvé, udržuje změny agenta kontrolovatelnými. Změna politiky je malá JSON modifikace, kterou lidský recenzent může rychle vyhodnotit. Zadruhé, udržuje analýzu reprodukovatelnou. Ostatní agenti a vývojáři spouštějící AigisCode na stejném projektu budou těžit z nahromaděných znalostí politik. Zatřetí, dodržuje princip nejmenšího oprávnění. Agent aktualizující soubory politik nemůže náhodně rozbít samotný analytický nástroj.</p>

<p>Soubor politiky podporuje bohatou sadu konfiguračních možností. Importní aliasy (<code>js_import_aliases</code>) říkají builderu grafu, jak resolovat aliasy cest jako <code>@/</code> v TypeScript projektech. Vzory vstupních bodů (<code>orphan_entry_patterns</code>) označují soubory, které jsou legitimními vstupními body, i když je nic neimportuje, jako CLI skripty nebo testovací fixtures. Vzory opuštění (<code>abandoned_entry_patterns</code>) říkají detektoru mrtvého kódu, které adresáře obsahují implementace rozhraní, které by neměly být označovány.</p>

<h2 id="practical-integration-examples">Praktické příklady integrace</h2>

<h3>Integrace do CI pipeline</h3>

<p>Nejběžnějším integračním vzorem je spouštění AigisCode v CI a zpracování výsledků agentem. CI pipeline spustí <code>aigiscode analyze .</code> na každém pull requestu. Pokud se ve srovnání se základní linií na hlavní větvi objeví nové nálezy, pipeline spustí agenta k přezkoumání nálezů, jejich klasifikaci a buď je automaticky opraví, nebo zanechá komentáře k PR.</p>

<pre><code># .github/workflows/code-quality.yml
- name: Run AigisCode Analysis
  run: aigiscode analyze .
- name: Compare with baseline
  run: diff .aigiscode/deterministic-analysis.json baseline-report.json
- name: Agent review (if new findings)
  run: agent review --report .aigiscode/deterministic-analysis.json</code></pre>

<h3>Plánovaná údržba</h3>

<p>Dalším vzorem je plánovaná údržba codebase. Agent spustí AigisCode týdně, přezkoumá kompletní report a vytvoří údržbový PR, který řeší nálezy s nejvyšší prioritou. To vytváří stálý rytmus strukturálního zlepšování bez nutnosti, aby vývojáři ručně třídili architektonické problémy.</p>

<h3>Onboardingové reporty</h3>

<p>Když nový vývojář nebo nový AI agent začne pracovat na codebase, spuštění AigisCode poskytne okamžitý strukturální přehled. Report ukazuje, kde jsou hotspoty závislostí, které moduly jsou nejvíce provázané a kde žije známý technický dluh. To je rychlejší a spolehlivější než čtení dokumentace, která může být zastaralá.</p>

<h2 id="the-future-of-ai-driven-maintenance">Budoucnost údržby řízené AI</h2>

<p>Směřujeme ke světu, kde je údržba codebase z velké části automatizovaná. AI agenti budou průběžně monitorovat metriky kvality kódu, detekovat degradaci a aplikovat opravy bez lidského zásahu u rutinních problémů. Lidští vývojáři se zaměří na architektonická rozhodnutí, produktový design a high-level vedení, které agenti potřebují k efektivní práci.</p>

<p>AigisCode je navržen pro tuto budoucnost. Jeho strukturovaný výstup, chování řízené politikami a jasné oddělení deterministické analýzy od AI klasifikace z něj dělají přirozený fit pro workflow řízené agenty. Nástroj poskytuje oči. Agent poskytuje ruce. A soubor politik poskytuje institucionální znalosti, které zajišťují efektivní spolupráci obou.</p>

<p>Týmy, které tyto nástroje integrují do svého workflow nyní, budou mít významnou výhodu. Nejen čistší kód, ale systém, který se časem stává chytřejším, jak politika hromadí znalosti specifické pro projekt a agenti se zlepšují v interpretaci a jednání na základě výsledků analýzy. Budoucnost kvality kódu není lepší linter. Je to inteligentní systém, kde analýza, politiky a autonomní agenti spolupracují na udržování zdraví codebase ve velkém měřítku.</p>
`,
      fr: `
<p>Le workflow de développement de 2026 ne ressemble en rien à ce que nous avions il y a cinq ans. Les agents de codage IA, des systèmes autonomes capables de lire, comprendre, modifier et tester du code, font désormais partie intégrante de l'ingénierie logicielle. Des agents comme Claude Code, GitHub Copilot Workspace et Codex peuvent implémenter des fonctionnalités à partir de descriptions d'issues, corriger des bogues à partir de logs d'erreurs et refactoriser du code en fonction d'objectifs architecturaux. Mais chaque agent partage un besoin commun : des informations structurées et fiables sur la base de code dans laquelle il travaille.</p>

<p>C'est là qu'AigisCode s'intègre dans le workflow des agents IA. Non pas comme une autre IA qui génère du code, mais comme la couche analytique qui donne aux agents le contexte nécessaire pour prendre de bonnes décisions sur la qualité du code et l'architecture.</p>

<h2 id="the-agent-workflow">Le workflow Analyser-Parser-Trier-Corriger</h2>

<p>Le workflow recommandé pour un agent IA utilisant AigisCode suit quatre étapes, chacune s'appuyant sur la précédente.</p>

<h3>Étape 1 : Analyser</h3>

<p>L'agent exécute <code>aigiscode analyze /path/to/project</code>. Cela lance le pipeline complet en six étapes : indexation des fichiers source, construction du graphe de dépendances, exécution des détecteurs, application des règles d'exclusion enregistrées, exécution de la revue IA pour la classification et génération du rapport. La sortie est un fichier JSON structuré dans <code>.aigiscode/deterministic-analysis.json</code>.</p>

<p>Cette étape est déterministe à l'exception de l'étape optionnelle de revue IA. Un agent peut exécuter avec <code>--skip-ai</code> pour une analyse entièrement déterministe, ce qui est utile dans les environnements CI où la reproductibilité importe plus qu'une classification nuancée.</p>

<h3>Étape 2 : Parser</h3>

<p>L'agent lit le rapport JSON et extrait les résultats structurés. Le rapport est organisé en sections claires qu'un agent peut naviguer de manière programmatique.</p>

<p><code>graph_analysis.strong_circular_dependencies</code> contient les cycles architecturaux nécessitant un refactoring structurel. Chaque cycle liste les modules participants et les arêtes d'import spécifiques formant la boucle. <code>graph_analysis.god_classes</code> identifie les classes avec un couplage et une responsabilité excessifs. <code>graph_analysis.bottleneck_files</code> met en évidence les fichiers situés sur trop de chemins de dépendances. <code>dead_code</code> catalogue les imports inutilisés, les méthodes non référencées, les classes abandonnées et les fichiers orphelins. <code>hardwiring</code> liste les chaînes magiques, les littéraux répétés et les adresses réseau codées en dur.</p>

<p>Chaque résultat inclut un niveau de confiance (élevé, moyen, faible) et une note de sévérité, donnant à l'agent suffisamment d'informations pour prendre des décisions de triage sans intervention humaine.</p>

<h3>Étape 3 : Trier</h3>

<p>C'est là que l'agent applique son jugement. Tous les résultats ne sont pas également importants, et tous les résultats ne sont pas de vrais positifs. Un workflow de triage d'agent bien conçu se présente ainsi.</p>

<p>Premièrement, filtrer par confiance. Commencez par les résultats à haute confiance. Ils ont le taux de faux positifs le plus bas et fournissent le signal le plus fiable sur les problèmes réels. Les résultats à confiance moyenne doivent être échantillonnés et vérifiés avant d'agir dessus en masse. Les résultats à faible confiance sont informationnels.</p>

<p>Deuxièmement, prioriser par impact. Une dépendance circulaire entre deux modules centraux que chaque fonctionnalité importe est plus impactante qu'une dépendance circulaire entre deux scripts utilitaires. Un résultat de code mort dans un module avec 50 dépendances entrantes compte plus que dans un helper de test isolé.</p>

<p>Troisièmement, classifier le type de correction. Certains résultats nécessitent de simples suppressions (imports inutilisés, fichiers orphelins). Certains nécessitent des changements d'interface (dépendances circulaires). Certains nécessitent des mises à jour de configuration (valeurs codées en dur). L'agent devrait regrouper les corrections similaires pour une exécution efficace.</p>

<h3>Étape 4 : Corriger</h3>

<p>L'agent applique les corrections en fonction des résultats du triage. Pour les corrections simples comme la suppression d'imports inutilisés ou la suppression de fichiers orphelins, l'agent peut agir avec une haute confiance. Pour les corrections structurelles comme la rupture de dépendances circulaires, l'agent devrait proposer un plan, l'implémenter et exécuter la suite de tests pour vérifier que le changement ne casse rien.</p>

<p>Après avoir appliqué les corrections, l'agent exécute <code>aigiscode analyze /path/to/project</code> pour régénérer le rapport à partir de l'index existant. C'est plus rapide qu'une analyse complète car l'étape d'indexation est sautée. L'agent peut alors comparer le nouveau rapport à la référence pour vérifier que les résultats ont été résolus et qu'aucun nouveau problème n'a été introduit.</p>

<h2 id="how-agents-consume-reports">Comment les agents réels consomment les rapports AigisCode</h2>

<p>Différents agents IA ont des forces différentes, et la façon dont ils consomment les rapports AigisCode reflète ces différences.</p>

<h3>Claude Code</h3>

<p>Les agents Claude Code excellent dans la compréhension du contexte et la prise de décisions nuancées. Lorsqu'on lui donne un rapport AigisCode, un agent Claude peut lire l'ensemble des résultats, comprendre les relations entre eux et développer un plan de refactoring cohérent qui traite plusieurs problèmes simultanément. Par exemple, si le rapport montre une dépendance circulaire entre les modules <code>auth</code> et <code>users</code> ainsi que du code mort dans les deux modules, Claude peut proposer un seul refactoring qui rompt le cycle et supprime le code mort en un changement cohérent.</p>

<p>Le format JSON structuré est particulièrement bien adapté à la fenêtre de contexte de Claude. L'agent peut charger l'intégralité du rapport, croiser les résultats avec le code source réel et produire des corrections qui tiennent compte du contexte complet de chaque problème.</p>

<h3>Agents Codex</h3>

<p>Les agents Codex sont efficaces pour exécuter des corrections ciblées dans un périmètre défini. Ils fonctionnent bien lorsqu'on leur donne un résultat spécifique du rapport AigisCode et qu'on leur demande de le corriger. Par exemple, pour un résultat de code mort concernant une méthode inutilisée spécifique, un agent Codex peut identifier tout le code associé, vérifier que la méthode est vraiment inutilisée et produire une suppression propre avec des imports mis à jour.</p>

<p>Le module workers d'AigisCode (<code>workers/codex.py</code>) fournit des points d'intégration spécifiquement conçus pour les agents de style Codex qui traitent les résultats un par un.</p>

<h2 id="policy-driven-behavior">Comportement d'agent piloté par les politiques</h2>

<p>L'un des principes de conception clés d'AigisCode est que le comportement spécifique au projet doit résider dans la politique, pas dans le code. Ce principe s'étend naturellement aux workflows des agents IA.</p>

<p>Lorsqu'un agent rencontre un faux positif, il ne doit pas modifier l'analyseur. Au lieu de cela, il doit ajouter une règle d'exclusion dans <code>.aigiscode/rules.json</code>. Lorsqu'il identifie un modèle de faux positifs (par exemple, toutes les méthodes d'un répertoire <code>Contracts/</code> sont signalées comme code mort parce qu'elles sont implémentées via des interfaces), il doit encoder le modèle dans <code>.aigiscode/policy.json</code>.</p>

<p>Cette approche présente plusieurs avantages. Premièrement, elle maintient les modifications de l'agent révisables. Un changement de politique est une petite modification JSON qu'un réviseur humain peut évaluer rapidement. Deuxièmement, elle maintient l'analyse reproductible. D'autres agents et développeurs exécutant AigisCode sur le même projet bénéficieront des connaissances de politique accumulées. Troisièmement, elle suit le principe du moindre privilège. Un agent mettant à jour les fichiers de politique ne peut pas accidentellement casser l'outil d'analyse lui-même.</p>

<p>Le fichier de politique prend en charge un riche ensemble d'options de configuration. Les alias d'import (<code>js_import_aliases</code>) indiquent au constructeur de graphe comment résoudre les alias de chemin comme <code>@/</code> dans les projets TypeScript. Les modèles de points d'entrée (<code>orphan_entry_patterns</code>) marquent les fichiers qui sont des points d'entrée légitimes même si rien ne les importe, comme les scripts CLI ou les fixtures de test. Les modèles d'abandon (<code>abandoned_entry_patterns</code>) indiquent au détecteur de code mort quels répertoires contiennent des implémentations d'interface qui ne doivent pas être signalées.</p>

<h2 id="practical-integration-examples">Exemples pratiques d'intégration</h2>

<h3>Intégration dans le pipeline CI</h3>

<p>Le modèle d'intégration le plus courant consiste à exécuter AigisCode dans le CI et à faire traiter les résultats par un agent. Le pipeline CI exécute <code>aigiscode analyze .</code> sur chaque pull request. Si de nouveaux résultats apparaissent par rapport à la référence sur la branche principale, le pipeline déclenche un agent pour réviser les résultats, les classifier et soit les corriger automatiquement, soit laisser des commentaires de revue sur la PR.</p>

<pre><code># .github/workflows/code-quality.yml
- name: Run AigisCode Analysis
  run: aigiscode analyze .
- name: Compare with baseline
  run: diff .aigiscode/deterministic-analysis.json baseline-report.json
- name: Agent review (if new findings)
  run: agent review --report .aigiscode/deterministic-analysis.json</code></pre>

<h3>Maintenance planifiée</h3>

<p>Un autre modèle est la maintenance planifiée de la base de code. Un agent exécute AigisCode chaque semaine, révise le rapport complet et crée une PR de maintenance qui traite les résultats de plus haute priorité. Cela crée un rythme régulier d'amélioration structurelle sans obliger les développeurs à trier manuellement les problèmes architecturaux.</p>

<h3>Rapports d'intégration</h3>

<p>Lorsqu'un nouveau développeur ou un nouvel agent IA commence à travailler sur une base de code, exécuter AigisCode fournit une vue d'ensemble structurelle instantanée. Le rapport montre où se trouvent les points chauds de dépendances, quels modules sont les plus couplés et où réside la dette technique connue. C'est plus rapide et plus fiable que la lecture d'une documentation qui peut être obsolète.</p>

<h2 id="the-future-of-ai-driven-maintenance">L'avenir de la maintenance pilotée par l'IA</h2>

<p>Nous nous dirigeons vers un monde où la maintenance des bases de code est largement automatisée. Les agents IA surveilleront continuellement les métriques de qualité du code, détecteront la dégradation et appliqueront des corrections sans intervention humaine pour les problèmes de routine. Les développeurs humains se concentreront sur les décisions architecturales, la conception produit et les orientations de haut niveau dont les agents ont besoin pour travailler efficacement.</p>

<p>AigisCode est conçu pour cet avenir. Sa sortie structurée, son comportement piloté par les politiques et sa séparation claire entre analyse déterministe et classification IA en font un ajustement naturel pour les workflows pilotés par des agents. L'outil fournit les yeux. L'agent fournit les mains. Et le fichier de politique fournit la connaissance institutionnelle qui garantit que les deux travaillent ensemble efficacement.</p>

<p>Les équipes qui intègrent ces outils dans leur workflow maintenant auront un avantage significatif. Pas seulement un code plus propre, mais un système qui devient plus intelligent au fil du temps à mesure que la politique accumule des connaissances spécifiques au projet et que les agents s'améliorent dans l'interprétation et l'action sur les résultats d'analyse. L'avenir de la qualité du code n'est pas un meilleur linter. C'est un système intelligent où analyse, politiques et agents autonomes travaillent ensemble pour maintenir les bases de code en bonne santé à grande échelle.</p>
`,
      es: `
<p>El workflow de desarrollo de 2026 no se parece en nada a lo que teníamos hace cinco años. Los agentes de codificación IA, sistemas autónomos que pueden leer, comprender, modificar y probar código, son ahora una parte habitual de la ingeniería de software. Agentes como Claude Code, GitHub Copilot Workspace y Codex pueden implementar funcionalidades a partir de descripciones de issues, corregir errores a partir de logs de errores y refactorizar código basándose en objetivos arquitectónicos. Pero cada agente comparte una necesidad común: información estructurada y fiable sobre la base de código en la que trabaja.</p>

<p>Aquí es donde AigisCode encaja en el workflow de los agentes IA. No como otra IA que genera código, sino como la capa analítica que da a los agentes el contexto que necesitan para tomar buenas decisiones sobre la calidad del código y la arquitectura.</p>

<h2 id="the-agent-workflow">El workflow Analizar-Parsear-Clasificar-Corregir</h2>

<p>El workflow recomendado para un agente IA que usa AigisCode sigue cuatro etapas, cada una construyendo sobre la anterior.</p>

<h3>Etapa 1: Analizar</h3>

<p>El agente ejecuta <code>aigiscode analyze /path/to/project</code>. Esto ejecuta el pipeline completo de seis etapas: indexación de archivos fuente, construcción del grafo de dependencias, ejecución de detectores, aplicación de reglas de exclusión guardadas, ejecución de revisión IA para clasificación y generación del informe. La salida es un archivo JSON estructurado en <code>.aigiscode/deterministic-analysis.json</code>.</p>

<p>Este paso es determinista excepto por la etapa opcional de revisión IA. Un agente puede ejecutar con <code>--skip-ai</code> para un análisis completamente determinista, lo cual es útil en entornos CI donde la reproducibilidad importa más que la clasificación matizada.</p>

<h3>Etapa 2: Parsear</h3>

<p>El agente lee el informe JSON y extrae hallazgos estructurados. El informe está organizado en secciones claras que un agente puede navegar programáticamente.</p>

<p><code>graph_analysis.strong_circular_dependencies</code> contiene los ciclos arquitectónicos que necesitan refactorización estructural. Cada ciclo lista los módulos participantes y las aristas de importación específicas que forman el bucle. <code>graph_analysis.god_classes</code> identifica clases con acoplamiento y responsabilidad excesivos. <code>graph_analysis.bottleneck_files</code> destaca archivos que se encuentran en demasiadas rutas de dependencias. <code>dead_code</code> cataloga importaciones no utilizadas, métodos no referenciados, clases abandonadas y archivos huérfanos. <code>hardwiring</code> lista cadenas mágicas, literales repetidos y direcciones de red codificadas en duro.</p>

<p>Cada hallazgo incluye un nivel de confianza (alto, medio, bajo) y una calificación de severidad, dando al agente suficiente información para tomar decisiones de clasificación sin intervención humana.</p>

<h3>Etapa 3: Clasificar</h3>

<p>Aquí es donde el agente aplica su juicio. No todos los hallazgos son igualmente importantes, y no todos los hallazgos son verdaderos positivos. Un workflow de clasificación de agente bien diseñado se ve así.</p>

<p>Primero, filtrar por confianza. Comience con los hallazgos de alta confianza. Estos tienen la tasa más baja de falsos positivos y proporcionan la señal más confiable sobre problemas reales. Los hallazgos de confianza media deben ser muestreados y verificados antes de actuar sobre ellos en masa. Los hallazgos de baja confianza son informativos.</p>

<p>Segundo, priorizar por impacto. Una dependencia circular entre dos módulos centrales que cada funcionalidad importa es más impactante que una dependencia circular entre dos scripts utilitarios. Un hallazgo de código muerto en un módulo con 50 dependencias entrantes importa más que uno en un helper de test aislado.</p>

<p>Tercero, clasificar el tipo de corrección. Algunos hallazgos requieren eliminaciones simples (importaciones no utilizadas, archivos huérfanos). Algunos requieren cambios de interfaz (dependencias circulares). Algunos requieren actualizaciones de configuración (valores codificados en duro). El agente debería agrupar correcciones similares para una ejecución eficiente.</p>

<h3>Etapa 4: Corregir</h3>

<p>El agente aplica correcciones basándose en los resultados de la clasificación. Para correcciones simples como eliminar importaciones no utilizadas o borrar archivos huérfanos, el agente puede actuar con alta confianza. Para correcciones estructurales como romper dependencias circulares, el agente debería proponer un plan, implementarlo y ejecutar la suite de tests para verificar que el cambio no rompe nada.</p>

<p>Después de aplicar las correcciones, el agente ejecuta <code>aigiscode analyze /path/to/project</code> para regenerar el informe desde el índice existente. Esto es más rápido que un análisis completo porque omite la etapa de indexación. El agente puede entonces comparar el nuevo informe con la línea base para verificar que los hallazgos se han resuelto y no se han introducido nuevos problemas.</p>

<h2 id="how-agents-consume-reports">Cómo los agentes reales consumen los informes de AigisCode</h2>

<p>Diferentes agentes IA tienen diferentes fortalezas, y la forma en que consumen los informes de AigisCode refleja esas diferencias.</p>

<h3>Claude Code</h3>

<p>Los agentes Claude Code sobresalen en la comprensión del contexto y la toma de decisiones matizadas. Cuando se le da un informe de AigisCode, un agente Claude puede leer todos los hallazgos, entender las relaciones entre ellos y desarrollar un plan de refactorización coherente que aborde múltiples problemas simultáneamente. Por ejemplo, si el informe muestra una dependencia circular entre los módulos <code>auth</code> y <code>users</code> y también código muerto en ambos módulos, Claude puede proponer una única refactorización que rompa el ciclo y elimine el código muerto en un cambio coherente.</p>

<p>El formato JSON estructurado es particularmente adecuado para la ventana de contexto de Claude. El agente puede cargar el informe completo, cruzar los hallazgos con el código fuente real y producir correcciones que consideren el contexto completo de cada problema.</p>

<h3>Agentes Codex</h3>

<p>Los agentes Codex son efectivos ejecutando correcciones dirigidas dentro de un alcance definido. Funcionan bien cuando se les da un hallazgo específico del informe de AigisCode y se les pide que lo corrijan. Por ejemplo, dado un hallazgo de código muerto para un método no utilizado específico, un agente Codex puede identificar todo el código relacionado, verificar que el método realmente no se usa y producir una eliminación limpia con importaciones actualizadas.</p>

<p>El módulo workers de AigisCode (<code>workers/codex.py</code>) proporciona puntos de integración específicamente diseñados para agentes estilo Codex que procesan hallazgos uno a la vez.</p>

<h2 id="policy-driven-behavior">Comportamiento de agente dirigido por políticas</h2>

<p>Uno de los principios de diseño clave de AigisCode es que el comportamiento específico del proyecto debe residir en las políticas, no en el código. Este principio se extiende naturalmente a los workflows de agentes IA.</p>

<p>Cuando un agente encuentra un falso positivo, no debe modificar el analizador. En su lugar, debe agregar una regla de exclusión en <code>.aigiscode/rules.json</code>. Cuando identifica un patrón de falsos positivos (por ejemplo, todos los métodos en un directorio <code>Contracts/</code> se marcan como código muerto porque se implementan vía interfaces), debe codificar el patrón en <code>.aigiscode/policy.json</code>.</p>

<p>Este enfoque tiene varias ventajas. Primero, mantiene los cambios del agente revisables. Un cambio de política es una pequeña modificación JSON que un revisor humano puede evaluar rápidamente. Segundo, mantiene el análisis reproducible. Otros agentes y otros desarrolladores ejecutando AigisCode en el mismo proyecto se beneficiarán del conocimiento de política acumulado. Tercero, sigue el principio de mínimo privilegio. Un agente actualizando archivos de política no puede romper accidentalmente la herramienta de análisis en sí.</p>

<p>El archivo de política soporta un rico conjunto de opciones de configuración. Los alias de importación (<code>js_import_aliases</code>) le dicen al constructor del grafo cómo resolver alias de ruta como <code>@/</code> en proyectos TypeScript. Los patrones de punto de entrada (<code>orphan_entry_patterns</code>) marcan archivos que son puntos de entrada legítimos aunque nada los importe, como scripts CLI o fixtures de test. Los patrones de abandono (<code>abandoned_entry_patterns</code>) le dicen al detector de código muerto qué directorios contienen implementaciones de interfaz que no deben ser marcadas.</p>

<h2 id="practical-integration-examples">Ejemplos prácticos de integración</h2>

<h3>Integración en pipeline CI</h3>

<p>El patrón de integración más común es ejecutar AigisCode en CI y hacer que un agente procese los resultados. El pipeline CI ejecuta <code>aigiscode analyze .</code> en cada pull request. Si aparecen nuevos hallazgos comparados con la línea base en la rama principal, el pipeline desencadena un agente para revisar los hallazgos, clasificarlos y ya sea corregirlos automáticamente o dejar comentarios de revisión en la PR.</p>

<pre><code># .github/workflows/code-quality.yml
- name: Run AigisCode Analysis
  run: aigiscode analyze .
- name: Compare with baseline
  run: diff .aigiscode/deterministic-analysis.json baseline-report.json
- name: Agent review (if new findings)
  run: agent review --report .aigiscode/deterministic-analysis.json</code></pre>

<h3>Mantenimiento programado</h3>

<p>Otro patrón es el mantenimiento programado de la base de código. Un agente ejecuta AigisCode semanalmente, revisa el informe completo y crea una PR de mantenimiento que aborda los hallazgos de mayor prioridad. Esto crea una cadencia constante de mejora estructural sin requerir que los desarrolladores clasifiquen manualmente los problemas arquitectónicos.</p>

<h3>Informes de incorporación</h3>

<p>Cuando un nuevo desarrollador o un nuevo agente IA comienza a trabajar en una base de código, ejecutar AigisCode proporciona una visión estructural instantánea. El informe muestra dónde están los puntos calientes de dependencias, qué módulos están más acoplados y dónde reside la deuda técnica conocida. Esto es más rápido y más confiable que leer documentación que puede estar desactualizada.</p>

<h2 id="the-future-of-ai-driven-maintenance">El futuro del mantenimiento impulsado por IA</h2>

<p>Nos dirigimos hacia un mundo donde el mantenimiento de las bases de código está en gran parte automatizado. Los agentes IA monitorearán continuamente las métricas de calidad del código, detectarán degradación y aplicarán correcciones sin intervención humana para problemas de rutina. Los desarrolladores humanos se enfocarán en decisiones arquitectónicas, diseño de producto y la orientación de alto nivel que los agentes necesitan para trabajar eficazmente.</p>

<p>AigisCode está diseñado para este futuro. Su salida estructurada, comportamiento dirigido por políticas y clara separación entre análisis determinista y clasificación IA lo hacen un ajuste natural para workflows dirigidos por agentes. La herramienta proporciona los ojos. El agente proporciona las manos. Y el archivo de política proporciona el conocimiento institucional que asegura que ambos trabajen juntos eficazmente.</p>

<p>Los equipos que integren estas herramientas en su workflow ahora tendrán una ventaja significativa. No solo código más limpio, sino un sistema que se vuelve más inteligente con el tiempo a medida que la política acumula conocimiento específico del proyecto y los agentes mejoran en la interpretación y acción sobre los resultados del análisis. El futuro de la calidad del código no es un mejor linter. Es un sistema inteligente donde análisis, políticas y agentes autónomos trabajan juntos para mantener las bases de código saludables a escala.</p>
`,
      zh: `
<p>2026 年的开发工作流与五年前完全不同。AI 编程代理——能够读取、理解、修改和测试代码的自主系统——现在是软件工程的常规组成部分。Claude Code、GitHub Copilot Workspace 和 Codex 等代理可以从 issue 描述实现功能、从错误日志修复 bug、根据架构目标重构代码。但每个代理都有一个共同需求：关于其工作代码库的结构化、可靠的信息。</p>

<p>这正是 AigisCode 融入 AI 代理工作流的位置。不是作为另一个生成代码的 AI，而是作为为代理提供做出良好代码质量和架构决策所需上下文的分析层。</p>

<h2 id="the-agent-workflow">分析-解析-分类-修复工作流</h2>

<p>使用 AigisCode 的 AI 代理推荐工作流遵循四个阶段，每个阶段都建立在前一个阶段之上。</p>

<h3>阶段 1：分析</h3>

<p>代理运行 <code>aigiscode analyze /path/to/project</code>。这将执行完整的六阶段流水线：索引源文件、构建依赖图、运行检测器、应用保存的排除规则、运行 AI 审查进行分类以及生成报告。输出是位于 <code>.aigiscode/deterministic-analysis.json</code> 的结构化 JSON 文件。</p>

<p>除了可选的 AI 审查阶段外，此步骤是确定性的。代理可以使用 <code>--skip-ai</code> 运行完全确定性的分析，这在可重复性比细微分类更重要的 CI 环境中很有用。</p>

<h3>阶段 2：解析</h3>

<p>代理读取 JSON 报告并提取结构化发现。报告被组织成清晰的部分，代理可以以编程方式导航。</p>

<p><code>graph_analysis.strong_circular_dependencies</code> 包含需要结构重构的架构循环。每个循环列出参与的模块和形成环路的特定导入边。<code>graph_analysis.god_classes</code> 识别具有过度耦合和责任的类。<code>graph_analysis.bottleneck_files</code> 突出显示位于过多依赖路径上的文件。<code>dead_code</code> 编目未使用的导入、未引用的方法、废弃的类和孤立文件。<code>hardwiring</code> 列出魔术字符串、重复的字面量和硬编码的网络地址。</p>

<p>每个发现都包含置信度级别（高、中、低）和严重性评级，为代理提供足够的信息来做出分类决策，无需人工输入。</p>

<h3>阶段 3：分类</h3>

<p>这是代理应用判断的地方。不是所有发现都同样重要，也不是所有发现都是真阳性。一个设计良好的代理分类工作流如下所示。</p>

<p>首先，按置信度过滤。从高置信度发现开始。这些发现的假阳性率最低，提供关于实际问题最可靠的信号。中等置信度的发现在批量处理之前应该先抽样验证。低置信度的发现是参考性的。</p>

<p>其次，按影响优先排序。两个核心模块之间的循环依赖（每个功能都导入这些模块）比两个工具脚本之间的循环依赖更具影响力。具有 50 个入站依赖的模块中的死代码发现比孤立测试辅助程序中的更重要。</p>

<p>第三，对修复类型进行分类。一些发现需要简单的删除（未使用的导入、孤立文件）。一些需要接口更改（循环依赖）。一些需要配置更新（硬编码值）。代理应该将类似的修复批量处理以提高效率。</p>

<h3>阶段 4：修复</h3>

<p>代理根据分类结果应用修复。对于简单的修复，如删除未使用的导入或删除孤立文件，代理可以高置信度地执行。对于结构性修复，如打破循环依赖，代理应该提出计划、实施它并运行测试套件以验证更改不会破坏任何东西。</p>

<p>应用修复后，代理运行 <code>aigiscode analyze /path/to/project</code> 从现有索引重新生成报告。这比完整分析更快，因为它跳过了索引阶段。然后代理可以将新报告与基线进行比较，以验证发现已被解决且没有引入新问题。</p>

<h2 id="how-agents-consume-reports">真实代理如何消费 AigisCode 报告</h2>

<p>不同的 AI 代理有不同的优势，它们消费 AigisCode 报告的方式反映了这些差异。</p>

<h3>Claude Code</h3>

<p>Claude Code 代理擅长理解上下文和做出细致的决策。当给定一个 AigisCode 报告时，Claude 代理可以阅读所有发现、理解它们之间的关系并制定一个同时解决多个问题的连贯重构计划。例如，如果报告显示 <code>auth</code> 和 <code>users</code> 模块之间存在循环依赖，并且两个模块中都有死代码，Claude 可以提出一个单一的重构方案，在一个连贯的变更中打破循环并删除死代码。</p>

<p>结构化的 JSON 格式特别适合 Claude 的上下文窗口。代理可以加载整个报告，将发现与实际源代码交叉引用，并生成考虑到每个问题完整上下文的修复。</p>

<h3>Codex 代理</h3>

<p>Codex 代理擅长在定义范围内执行有针对性的修复。当给定 AigisCode 报告中的特定发现并要求修复时，它们工作得很好。例如，对于特定未使用方法的死代码发现，Codex 代理可以识别所有相关代码、验证该方法确实未被使用，并生成带有更新导入的干净删除。</p>

<p>AigisCode 的 workers 模块（<code>workers/codex.py</code>）提供了专为 Codex 风格代理设计的集成点，这些代理一次处理一个发现。</p>

<h2 id="policy-driven-behavior">策略驱动的代理行为</h2>

<p>AigisCode 的关键设计原则之一是项目特定的行为应该存在于策略中，而不是代码中。这一原则自然地延伸到 AI 代理工作流。</p>

<p>当代理遇到假阳性时，它不应该修改分析器。相反，它应该在 <code>.aigiscode/rules.json</code> 中添加排除规则。当它识别到假阳性模式时（例如，<code>Contracts/</code> 目录中的所有方法因为通过接口实现而被标记为死代码），它应该在 <code>.aigiscode/policy.json</code> 中编码该模式。</p>

<p>这种方法有几个优势。首先，它使代理的更改可审查。策略更改是一个小的 JSON 修改，人工审查者可以快速评估。其次，它保持分析的可重复性。在同一项目上运行 AigisCode 的其他代理和开发者将受益于积累的策略知识。第三，它遵循最小权限原则。更新策略文件的代理不会意外破坏分析工具本身。</p>

<p>策略文件支持丰富的配置选项集。导入别名（<code>js_import_aliases</code>）告诉图构建器如何解析 TypeScript 项目中的路径别名，如 <code>@/</code>。入口点模式（<code>orphan_entry_patterns</code>）标记那些虽然没有被导入但属于合法入口点的文件，如 CLI 脚本或测试 fixture。废弃模式（<code>abandoned_entry_patterns</code>）告诉死代码检测器哪些目录包含不应被标记的接口实现。</p>

<h2 id="practical-integration-examples">实际集成示例</h2>

<h3>CI 流水线集成</h3>

<p>最常见的集成模式是在 CI 中运行 AigisCode 并让代理处理结果。CI 流水线在每个 pull request 上运行 <code>aigiscode analyze .</code>。如果与主分支的基线相比出现新发现，流水线会触发代理审查发现、分类它们，然后自动修复或在 PR 上留下审查评论。</p>

<pre><code># .github/workflows/code-quality.yml
- name: Run AigisCode Analysis
  run: aigiscode analyze .
- name: Compare with baseline
  run: diff .aigiscode/deterministic-analysis.json baseline-report.json
- name: Agent review (if new findings)
  run: agent review --report .aigiscode/deterministic-analysis.json</code></pre>

<h3>定期维护</h3>

<p>另一种模式是定期代码库维护。代理每周运行 AigisCode，审查完整报告，并创建一个处理最高优先级发现的维护 PR。这创造了结构改进的稳定节奏，无需开发者手动分类架构问题。</p>

<h3>入职报告</h3>

<p>当新开发者或新 AI 代理开始在代码库上工作时，运行 AigisCode 提供即时的结构概览。报告显示依赖热点在哪里、哪些模块耦合最紧密以及已知技术债务在哪里。这比阅读可能过时的文档更快、更可靠。</p>

<h2 id="the-future-of-ai-driven-maintenance">AI 驱动维护的未来</h2>

<p>我们正走向一个代码库维护在很大程度上自动化的世界。AI 代理将持续监控代码质量指标、检测退化，并在无需人工干预的情况下对常规问题应用修复。人类开发者将专注于架构决策、产品设计以及代理有效工作所需的高层指导。</p>

<p>AigisCode 就是为这个未来设计的。它的结构化输出、策略驱动行为以及确定性分析与 AI 分类之间的清晰分离使其天然适合代理驱动的工作流。工具提供眼睛。代理提供双手。策略文件提供确保两者有效协作的制度知识。</p>

<p>现在将这些工具集成到工作流中的团队将拥有显著优势。不仅是更干净的代码，而是一个随着时间推移变得更智能的系统——策略积累项目特定知识，代理在解释和处理分析结果方面不断改进。代码质量的未来不是更好的 linter，而是一个智能系统，分析、策略和自主代理协同工作，大规模保持代码库的健康。</p>
`,
      hi: `

<p>2026 का विकास वर्कफ़्लो पांच साल पहले जो हमारे पास था उससे बिल्कुल अलग दिखता है। AI कोडिंग एजेंट, स्वायत्त प्रणालियां जो कोड पढ़, समझ, संशोधित और परीक्षण कर सकती हैं, अब सॉफ्टवेयर इंजीनियरिंग का नियमित हिस्सा हैं। Claude Code, GitHub Copilot Workspace और Codex जैसे एजेंट इश्यू विवरणों से फीचर्स लागू कर सकते हैं, एरर लॉग से बग ठीक कर सकते हैं, और आर्किटेक्चरल लक्ष्यों के आधार पर कोड रीफैक्टर कर सकते हैं। लेकिन हर एजेंट की एक सामान्य आवश्यकता है: उस कोडबेस के बारे में संरचित, विश्वसनीय जानकारी जिसमें वे काम कर रहे हैं।</p>

<p>यहीं पर AigisCode AI एजेंट वर्कफ़्लो में फिट होता है। एक और AI के रूप में नहीं जो कोड उत्पन्न करता है, बल्कि विश्लेषणात्मक परत के रूप में जो एजेंट्स को कोड गुणवत्ता और आर्किटेक्चर के बारे में अच्छे निर्णय लेने के लिए आवश्यक संदर्भ देती है।</p>

<h2 id="the-agent-workflow">Analyze-Parse-Triage-Fix वर्कफ़्लो</h2>

<p>AigisCode का उपयोग करने वाले AI एजेंट के लिए अनुशंसित वर्कफ़्लो चार चरणों का पालन करता है, प्रत्येक पिछले पर आधारित है।</p>

<h3>चरण 1: विश्लेषण</h3>

<p>एजेंट <code>aigiscode analyze /path/to/project</code> चलाता है। यह पूर्ण छह-चरणीय पाइपलाइन निष्पादित करता है: स्रोत फाइलों की अनुक्रमणिका, डिपेंडेंसी ग्राफ का निर्माण, डिटेक्टर चलाना, सहेजे गए बहिष्करण नियम लागू करना, वर्गीकरण के लिए AI समीक्षा चलाना, और रिपोर्ट उत्पन्न करना। आउटपुट <code>.aigiscode/deterministic-analysis.json</code> पर एक संरचित JSON फाइल है।</p>

<p>यह कदम वैकल्पिक AI समीक्षा चरण को छोड़कर नियतात्मक है। एजेंट पूरी तरह से नियतात्मक विश्लेषण के लिए <code>--skip-ai</code> के साथ चला सकता है, जो CI वातावरण में उपयोगी है जहां पुनरुत्पादनीयता सूक्ष्म वर्गीकरण से अधिक महत्वपूर्ण है।</p>

<h3>चरण 2: पार्स</h3>

<p>एजेंट JSON रिपोर्ट पढ़ता है और संरचित निष्कर्ष निकालता है। रिपोर्ट स्पष्ट खंडों में व्यवस्थित है जिन्हें एजेंट प्रोग्रामेटिक रूप से नेविगेट कर सकता है।</p>

<p><code>graph_analysis.strong_circular_dependencies</code> में आर्किटेक्चरल चक्र हैं जिन्हें संरचनात्मक रीफैक्टरिंग की आवश्यकता है। प्रत्येक चक्र भाग लेने वाले मॉड्यूल और लूप बनाने वाले विशिष्ट आयात किनारों को सूचीबद्ध करता है। <code>graph_analysis.god_classes</code> अत्यधिक युग्मन और जिम्मेदारी वाली क्लासेज की पहचान करता है। <code>graph_analysis.bottleneck_files</code> उन फाइलों को हाइलाइट करता है जो बहुत सारे डिपेंडेंसी पथों पर बैठी हैं। <code>dead_code</code> अनुपयोगी आयात, अनसंदर्भित विधियों, परित्यक्त क्लासेज और अनाथ फाइलों को सूचीबद्ध करता है। <code>hardwiring</code> मैजिक स्ट्रिंग्स, दोहराए गए लिटरल्स और हार्डकोडेड नेटवर्क पतों को सूचीबद्ध करता है।</p>

<p>प्रत्येक निष्कर्ष में एक विश्वास स्तर (उच्च, मध्यम, निम्न) और एक गंभीरता रेटिंग शामिल है, जो एजेंट को मानव इनपुट के बिना ट्राइएज निर्णय लेने के लिए पर्याप्त जानकारी देती है।</p>

<h3>चरण 3: ट्राइएज</h3>

<p>यहां एजेंट निर्णय लागू करता है। सभी निष्कर्ष समान रूप से महत्वपूर्ण नहीं हैं, और सभी निष्कर्ष सही सकारात्मक नहीं हैं। एक अच्छी तरह से डिज़ाइन किया गया एजेंट ट्राइएज वर्कफ़्लो इस तरह दिखता है।</p>

<p>पहला, विश्वास के अनुसार फ़िल्टर करें। उच्च-विश्वास निष्कर्षों से शुरू करें। इनमें सबसे कम गलत-सकारात्मक दर है और वास्तविक मुद्दों के बारे में सबसे विश्वसनीय संकेत प्रदान करते हैं। मध्यम-विश्वास निष्कर्षों को थोक में कार्य करने से पहले नमूना और सत्यापित किया जाना चाहिए। निम्न-विश्वास निष्कर्ष सूचनात्मक हैं।</p>

<p>दूसरा, प्रभाव के अनुसार प्राथमिकता दें। दो कोर मॉड्यूलों के बीच चक्रीय निर्भरता जिसे हर फीचर आयात करता है, दो यूटिलिटी स्क्रिप्ट्स के बीच चक्रीय निर्भरता से अधिक प्रभावशाली है। 50 इनबाउंड डिपेंडेंसी वाले मॉड्यूल में डेड कोड निष्कर्ष एक अलग टेस्ट हेल्पर में एक से अधिक मायने रखता है।</p>

<p>तीसरा, फिक्स प्रकार को वर्गीकृत करें। कुछ निष्कर्षों को सरल हटाने की आवश्यकता होती है (अनुपयोगी आयात, अनाथ फाइलें)। कुछ को इंटरफेस परिवर्तन की आवश्यकता होती है (चक्रीय निर्भरताएं)। कुछ को कॉन्फ़िगरेशन अपडेट की आवश्यकता होती है (हार्डवायर्ड मान)। एजेंट को कुशल निष्पादन के लिए समान फिक्स को एक साथ बैच करना चाहिए।</p>

<h3>चरण 4: फिक्स</h3>

<p>एजेंट ट्राइएज परिणामों के आधार पर फिक्स लागू करता है। अनुपयोगी आयात हटाने या अनाथ फाइलें डिलीट करने जैसे सरल फिक्स के लिए, एजेंट उच्च विश्वास के साथ कार्य कर सकता है। चक्रीय निर्भरताओं को तोड़ने जैसे संरचनात्मक फिक्स के लिए, एजेंट को एक योजना प्रस्तावित करनी चाहिए, इसे लागू करना चाहिए, और यह सत्यापित करने के लिए टेस्ट सूट चलाना चाहिए कि परिवर्तन कुछ भी नहीं तोड़ता।</p>

<p>फिक्स लागू करने के बाद, एजेंट मौजूदा इंडेक्स से रिपोर्ट पुनः उत्पन्न करने के लिए <code>aigiscode analyze /path/to/project</code> चलाता है। यह पूर्ण विश्लेषण से तेज है क्योंकि यह अनुक्रमणिका चरण को छोड़ देता है। एजेंट तब नई रिपोर्ट की तुलना बेसलाइन से कर सकता है यह सत्यापित करने के लिए कि निष्कर्ष हल हो गए हैं और कोई नए मुद्दे पेश नहीं हुए हैं।</p>

<h2 id="how-agents-consume-reports">वास्तविक एजेंट AigisCode रिपोर्ट का उपभोग कैसे करते हैं</h2>

<p>विभिन्न AI एजेंट्स की विभिन्न ताकतें हैं, और जिस तरह से वे AigisCode रिपोर्ट का उपभोग करते हैं वह उन अंतरों को दर्शाता है।</p>

<h3>Claude Code</h3>

<p>Claude Code एजेंट्स संदर्भ समझने और सूक्ष्म निर्णय लेने में उत्कृष्ट हैं। AigisCode रिपोर्ट दिए जाने पर, एक Claude एजेंट पूर्ण निष्कर्ष पढ़ सकता है, उनके बीच संबंधों को समझ सकता है, और एक सुसंगत रीफैक्टरिंग योजना विकसित कर सकता है जो एक साथ कई मुद्दों को संबोधित करती है। उदाहरण के लिए, यदि रिपोर्ट <code>auth</code> और <code>users</code> मॉड्यूलों के बीच चक्रीय निर्भरता और दोनों मॉड्यूलों में डेड कोड दिखाती है, तो Claude एक ही सुसंगत रीफैक्टरिंग प्रस्तावित कर सकता है जो चक्र को तोड़ती है और एक सुसंगत परिवर्तन में डेड कोड को हटाती है।</p>

<p>संरचित JSON प्रारूप विशेष रूप से Claude के संदर्भ विंडो के लिए उपयुक्त है। एजेंट पूरी रिपोर्ट लोड कर सकता है, वास्तविक स्रोत कोड के साथ निष्कर्षों को क्रॉस-रेफरेंस कर सकता है, और ऐसे फिक्स उत्पन्न कर सकता है जो प्रत्येक मुद्दे के पूर्ण संदर्भ को ध्यान में रखते हैं।</p>

<h3>Codex एजेंट्स</h3>

<p>Codex एजेंट्स एक परिभाषित दायरे के भीतर लक्षित फिक्स निष्पादित करने में प्रभावी हैं। वे तब अच्छा काम करते हैं जब उन्हें AigisCode रिपोर्ट से एक विशिष्ट निष्कर्ष दिया जाता है और इसे ठीक करने के लिए कहा जाता है। उदाहरण के लिए, एक विशिष्ट अनुपयोगी विधि के लिए डेड कोड निष्कर्ष दिए जाने पर, एक Codex एजेंट सभी संबंधित कोड की पहचान कर सकता है, सत्यापित कर सकता है कि विधि वास्तव में अनुपयोगी है, और अपडेटेड आयातों के साथ एक स्वच्छ हटाव उत्पन्न कर सकता है।</p>

<p>AigisCode वर्कर्स मॉड्यूल (<code>workers/codex.py</code>) विशेष रूप से Codex-शैली एजेंट्स के लिए डिज़ाइन किए गए एकीकरण बिंदु प्रदान करता है जो एक समय में एक निष्कर्ष को संसाधित करते हैं।</p>

<h2 id="policy-driven-behavior">नीति-चालित एजेंट व्यवहार</h2>

<p>AigisCode के प्रमुख डिज़ाइन सिद्धांतों में से एक यह है कि प्रोजेक्ट-विशिष्ट व्यवहार नीति में होना चाहिए, कोड में नहीं। यह सिद्धांत स्वाभाविक रूप से AI एजेंट वर्कफ़्लो तक विस्तारित होता है।</p>

<p>जब एक एजेंट को गलत सकारात्मक का सामना होता है, तो उसे विश्लेषक को संशोधित नहीं करना चाहिए। इसके बजाय, उसे <code>.aigiscode/rules.json</code> में एक बहिष्करण नियम जोड़ना चाहिए। जब यह गलत सकारात्मक का एक पैटर्न पहचानता है (उदाहरण के लिए, <code>Contracts/</code> डायरेक्टरी में सभी विधियां डेड कोड के रूप में चिह्नित हैं क्योंकि वे इंटरफेस के माध्यम से कार्यान्वित हैं), तो इसे <code>.aigiscode/policy.json</code> में पैटर्न एन्कोड करना चाहिए।</p>

<p>इस दृष्टिकोण के कई फायदे हैं। पहला, यह एजेंट के परिवर्तनों को समीक्षा योग्य रखता है। एक नीति परिवर्तन एक छोटा JSON संशोधन है जिसका मानव समीक्षक जल्दी मूल्यांकन कर सकता है। दूसरा, यह विश्लेषण को पुनरुत्पादनीय रखता है। उसी प्रोजेक्ट पर AigisCode चलाने वाले अन्य एजेंट और अन्य डेवलपर संचित नीति ज्ञान से लाभान्वित होंगे। तीसरा, यह न्यूनतम विशेषाधिकार के सिद्धांत का पालन करता है। नीति फाइलों को अपडेट करने वाला एजेंट गलती से विश्लेषण उपकरण को तोड़ नहीं सकता।</p>

<p>नीति फाइल कॉन्फ़िगरेशन विकल्पों का एक समृद्ध सेट समर्थन करती है। आयात उपनाम (<code>js_import_aliases</code>) ग्राफ बिल्डर को बताते हैं कि TypeScript प्रोजेक्ट्स में <code>@/</code> जैसे पथ उपनामों को कैसे हल करना है। एंट्री पॉइंट पैटर्न (<code>orphan_entry_patterns</code>) उन फाइलों को चिह्नित करते हैं जो वैध एंट्री पॉइंट हैं भले ही कुछ भी उन्हें आयात न करे, जैसे CLI स्क्रिप्ट या टेस्ट फिक्सचर। परित्याग पैटर्न (<code>abandoned_entry_patterns</code>) डेड कोड डिटेक्टर को बताते हैं कि किन डायरेक्ट्रीज में इंटरफेस कार्यान्वयन हैं जिन्हें चिह्नित नहीं किया जाना चाहिए।</p>

<h2 id="practical-integration-examples">व्यावहारिक एकीकरण उदाहरण</h2>

<h3>CI पाइपलाइन एकीकरण</h3>

<p>सबसे आम एकीकरण पैटर्न CI में AigisCode चलाना और एक एजेंट से परिणामों को संसाधित कराना है। CI पाइपलाइन हर पुल रिक्वेस्ट पर <code>aigiscode analyze .</code> चलाती है। यदि मुख्य शाखा पर बेसलाइन की तुलना में नए निष्कर्ष दिखाई देते हैं, तो पाइपलाइन एक एजेंट को निष्कर्षों की समीक्षा करने, उन्हें वर्गीकृत करने, और या तो उन्हें स्वचालित रूप से ठीक करने या PR पर समीक्षा टिप्पणियां छोड़ने के लिए ट्रिगर करती है।</p>

<pre><code># .github/workflows/code-quality.yml
- name: Run AigisCode Analysis
  run: aigiscode analyze .
- name: Compare with baseline
  run: diff .aigiscode/deterministic-analysis.json baseline-report.json
- name: Agent review (if new findings)
  run: agent review --report .aigiscode/deterministic-analysis.json</code></pre>

<h3>अनुसूचित रखरखाव</h3>

<p>एक अन्य पैटर्न अनुसूचित कोडबेस रखरखाव है। एक एजेंट साप्ताहिक रूप से AigisCode चलाता है, पूरी रिपोर्ट की समीक्षा करता है, और एक रखरखाव PR बनाता है जो उच्चतम-प्राथमिकता वाले निष्कर्षों को संबोधित करता है। यह डेवलपर्स से आर्किटेक्चरल मुद्दों को मैन्युअल रूप से ट्राइएज करने की आवश्यकता के बिना संरचनात्मक सुधार की एक स्थिर ताल बनाता है।</p>

<h3>ऑनबोर्डिंग रिपोर्ट</h3>

<p>जब कोई नया डेवलपर या नया AI एजेंट किसी कोडबेस पर काम करना शुरू करता है, तो AigisCode चलाना तत्काल संरचनात्मक अवलोकन प्रदान करता है। रिपोर्ट दिखाती है कि डिपेंडेंसी हॉटस्पॉट कहां हैं, कौन से मॉड्यूल सबसे अधिक युग्मित हैं, और ज्ञात तकनीकी ऋण कहां रहता है। यह उस दस्तावेज़ीकरण को पढ़ने से तेज और अधिक विश्वसनीय है जो पुराना हो सकता है।</p>

<h2 id="the-future-of-ai-driven-maintenance">AI-चालित रखरखाव का भविष्य</h2>

<p>हम ऐसी दुनिया की ओर बढ़ रहे हैं जहां कोडबेस रखरखाव काफी हद तक स्वचालित है। AI एजेंट लगातार कोड गुणवत्ता मेट्रिक्स की निगरानी करेंगे, गिरावट का पता लगाएंगे, और नियमित मुद्दों के लिए मानव हस्तक्षेप के बिना फिक्स लागू करेंगे। मानव डेवलपर आर्किटेक्चरल निर्णयों, उत्पाद डिज़ाइन, और उच्च-स्तरीय मार्गदर्शन पर ध्यान केंद्रित करेंगे जो एजेंट्स को प्रभावी ढंग से काम करने के लिए चाहिए।</p>

<p>AigisCode इस भविष्य के लिए डिज़ाइन किया गया है। इसका संरचित आउटपुट, नीति-चालित व्यवहार, और नियतात्मक विश्लेषण और AI वर्गीकरण के बीच स्पष्ट पृथक्करण इसे एजेंट-चालित वर्कफ़्लो के लिए स्वाभाविक रूप से उपयुक्त बनाता है। उपकरण आंखें प्रदान करता है। एजेंट हाथ प्रदान करता है। और नीति फाइल संस्थागत ज्ञान प्रदान करती है जो सुनिश्चित करता है कि दोनों प्रभावी ढंग से एक साथ काम करें।</p>

<p>जो टीमें अभी इन उपकरणों को अपने वर्कफ़्लो में एकीकृत करती हैं उन्हें एक महत्वपूर्ण लाभ होगा। केवल स्वच्छ कोड ही नहीं, बल्कि एक ऐसी प्रणाली जो समय के साथ होशियार होती जाती है जैसे-जैसे नीति प्रोजेक्ट-विशिष्ट ज्ञान जमा करती है और एजेंट विश्लेषण परिणामों की व्याख्या और कार्रवाई में बेहतर होते जाते हैं। कोड गुणवत्ता का भविष्य एक बेहतर लिंटर नहीं है। यह एक बुद्धिमान प्रणाली है जहां विश्लेषण, नीति, और स्वायत्त एजेंट बड़े पैमाने पर कोडबेस को स्वस्थ रखने के लिए एक साथ काम करते हैं।</p>
`,
      pt: `

<p>O fluxo de trabalho de desenvolvimento de 2026 não se parece em nada com o que tínhamos cinco anos atrás. Agentes de codificação com IA, sistemas autônomos que podem ler, compreender, modificar e testar código, agora são parte regular da engenharia de software. Agentes como Claude Code, GitHub Copilot Workspace e Codex podem implementar funcionalidades a partir de descrições de issues, corrigir bugs a partir de logs de erro e refatorar código com base em objetivos arquiteturais. Mas todo agente compartilha uma necessidade comum: informações estruturadas e confiáveis sobre a base de código em que estão trabalhando.</p>

<p>É aqui que o AigisCode se encaixa no fluxo de trabalho do agente de IA. Não como outra IA que gera código, mas como a camada analítica que fornece aos agentes o contexto necessário para tomar boas decisões sobre qualidade de código e arquitetura.</p>

<h2 id="the-agent-workflow">O Fluxo Analyze-Parse-Triage-Fix</h2>

<p>O fluxo de trabalho recomendado para um agente de IA usando o AigisCode segue quatro estágios, cada um construído sobre o anterior.</p>

<h3>Estágio 1: Analisar</h3>

<p>O agente executa <code>aigiscode analyze /path/to/project</code>. Isso executa o pipeline completo de seis estágios: indexação de arquivos fonte, construção do grafo de dependências, execução de detectores, aplicação de regras de exclusão salvas, execução de revisão por IA para classificação e geração do relatório. A saída é um arquivo JSON estruturado em <code>.aigiscode/deterministic-analysis.json</code>.</p>

<p>Este passo é determinístico exceto pelo estágio opcional de revisão por IA. Um agente pode executar com <code>--skip-ai</code> para análise totalmente determinística, o que é útil em ambientes de CI onde a reprodutibilidade importa mais que a classificação nuanceada.</p>

<h3>Estágio 2: Analisar Relatório</h3>

<p>O agente lê o relatório JSON e extrai descobertas estruturadas. O relatório é organizado em seções claras que um agente pode navegar programaticamente.</p>

<p><code>graph_analysis.strong_circular_dependencies</code> contém os ciclos arquiteturais que precisam de refatoração estrutural. Cada ciclo lista os módulos participantes e as arestas de importação específicas que formam o laço. <code>graph_analysis.god_classes</code> identifica classes com acoplamento e responsabilidade excessivos. <code>graph_analysis.bottleneck_files</code> destaca arquivos que estão em muitos caminhos de dependência. <code>dead_code</code> cataloga importações não utilizadas, métodos não referenciados, classes abandonadas e arquivos órfãos. <code>hardwiring</code> lista strings mágicas, literais repetidos e endereços de rede hardcoded.</p>

<p>Cada descoberta inclui um nível de confiança (alto, médio, baixo) e uma classificação de severidade, dando ao agente informação suficiente para tomar decisões de triagem sem input humano.</p>

<h3>Estágio 3: Triagem</h3>

<p>É aqui que o agente aplica julgamento. Nem todas as descobertas são igualmente importantes, e nem todas são verdadeiros positivos. Um fluxo de triagem de agente bem projetado funciona assim.</p>

<p>Primeiro, filtre por confiança. Comece com descobertas de alta confiança. Estas têm a menor taxa de falsos positivos e fornecem o sinal mais confiável sobre problemas reais. Descobertas de confiança média devem ser amostradas e verificadas antes de agir sobre elas em massa. Descobertas de baixa confiança são informacionais.</p>

<p>Segundo, priorize por impacto. Uma dependência circular entre dois módulos centrais que cada funcionalidade importa é mais impactante que uma dependência circular entre dois scripts utilitários. Uma descoberta de código morto em um módulo com 50 dependências de entrada importa mais que uma em um helper de teste isolado.</p>

<p>Terceiro, classifique o tipo de correção. Algumas descobertas requerem exclusões simples (importações não utilizadas, arquivos órfãos). Algumas requerem mudanças de interface (dependências circulares). Algumas requerem atualizações de configuração (valores hardcoded). O agente deve agrupar correções semelhantes para execução eficiente.</p>

<h3>Estágio 4: Corrigir</h3>

<p>O agente aplica correções com base nos resultados da triagem. Para correções simples como remover importações não utilizadas ou excluir arquivos órfãos, o agente pode agir com alta confiança. Para correções estruturais como quebrar dependências circulares, o agente deve propor um plano, implementá-lo e executar o conjunto de testes para verificar que a mudança não quebra nada.</p>

<p>Após aplicar correções, o agente executa <code>aigiscode analyze /path/to/project</code> para regenerar o relatório a partir do índice existente. Isso é mais rápido que uma análise completa porque pula o estágio de indexação. O agente pode então comparar o novo relatório com a linha de base para verificar que as descobertas foram resolvidas e nenhum novo problema foi introduzido.</p>

<h2 id="how-agents-consume-reports">Como Agentes Reais Consomem Relatórios do AigisCode</h2>

<p>Diferentes agentes de IA têm diferentes pontos fortes, e a forma como consomem relatórios do AigisCode reflete essas diferenças.</p>

<h3>Claude Code</h3>

<p>Agentes Claude Code se destacam em compreender contexto e tomar decisões nuanceadas. Quando recebem um relatório do AigisCode, um agente Claude pode ler todas as descobertas, compreender as relações entre elas e desenvolver um plano de refatoração coerente que aborda múltiplos problemas simultaneamente. Por exemplo, se o relatório mostra uma dependência circular entre os módulos <code>auth</code> e <code>users</code> e também código morto em ambos os módulos, Claude pode propor uma única refatoração que quebra o ciclo e remove o código morto em uma mudança coerente.</p>

<p>O formato JSON estruturado é particularmente adequado para a janela de contexto do Claude. O agente pode carregar o relatório inteiro, cruzar referências de descobertas com o código fonte real e produzir correções que levam em conta o contexto completo de cada problema.</p>

<h3>Agentes Codex</h3>

<p>Agentes Codex são eficazes na execução de correções direcionadas dentro de um escopo definido. Funcionam bem quando recebem uma descoberta específica do relatório do AigisCode e são solicitados a corrigi-la. Por exemplo, dada uma descoberta de código morto para um método não utilizado específico, um agente Codex pode identificar todo o código relacionado, verificar que o método é realmente não utilizado e produzir uma exclusão limpa com importações atualizadas.</p>

<p>O módulo de workers do AigisCode (<code>workers/codex.py</code>) fornece pontos de integração especificamente projetados para agentes estilo Codex que processam descobertas uma de cada vez.</p>

<h2 id="policy-driven-behavior">Comportamento de Agente Orientado por Políticas</h2>

<p>Um dos princípios-chave de design do AigisCode é que o comportamento específico do projeto deve residir em política, não em código. Este princípio se estende naturalmente aos fluxos de trabalho de agentes de IA.</p>

<p>Quando um agente encontra um falso positivo, não deve modificar o analisador. Em vez disso, deve adicionar uma regra de exclusão em <code>.aigiscode/rules.json</code>. Quando identifica um padrão de falsos positivos (por exemplo, todos os métodos num diretório <code>Contracts/</code> são sinalizados como código morto porque são implementados via interfaces), deve codificar o padrão em <code>.aigiscode/policy.json</code>.</p>

<p>Esta abordagem tem várias vantagens. Primeiro, mantém as mudanças do agente revisáveis. Uma mudança de política é uma pequena modificação JSON que um revisor humano pode avaliar rapidamente. Segundo, mantém a análise reprodutível. Outros agentes e outros desenvolvedores executando o AigisCode no mesmo projeto se beneficiarão do conhecimento de política acumulado. Terceiro, segue o princípio do menor privilégio. Um agente atualizando arquivos de política não pode acidentalmente quebrar a ferramenta de análise em si.</p>

<p>O arquivo de política suporta um conjunto rico de opções de configuração. Aliases de importação (<code>js_import_aliases</code>) dizem ao construtor de grafo como resolver aliases de caminho como <code>@/</code> em projetos TypeScript. Padrões de ponto de entrada (<code>orphan_entry_patterns</code>) marcam arquivos que são pontos de entrada legítimos mesmo que nada os importe, como scripts CLI ou fixtures de teste. Padrões de abandono (<code>abandoned_entry_patterns</code>) dizem ao detector de código morto quais diretórios contêm implementações de interface que não devem ser sinalizadas.</p>

<h2 id="practical-integration-examples">Exemplos Práticos de Integração</h2>

<h3>Integração com Pipeline de CI</h3>

<p>O padrão de integração mais comum é executar o AigisCode no CI e ter um agente processando os resultados. O pipeline de CI executa <code>aigiscode analyze .</code> em cada pull request. Se novas descobertas aparecerem comparadas à linha de base na branch principal, o pipeline aciona um agente para revisar as descobertas, classificá-las e corrigi-las automaticamente ou deixar comentários de revisão no PR.</p>

<pre><code># .github/workflows/code-quality.yml
- name: Run AigisCode Analysis
  run: aigiscode analyze .
- name: Compare with baseline
  run: diff .aigiscode/deterministic-analysis.json baseline-report.json
- name: Agent review (if new findings)
  run: agent review --report .aigiscode/deterministic-analysis.json</code></pre>

<h3>Manutenção Programada</h3>

<p>Outro padrão é manutenção programada da base de código. Um agente executa o AigisCode semanalmente, revisa o relatório completo e cria um PR de manutenção que aborda as descobertas de maior prioridade. Isso cria uma cadência constante de melhoria estrutural sem exigir que os desenvolvedores triagem manualmente problemas arquiteturais.</p>

<h3>Relatórios de Integração</h3>

<p>Quando um novo desenvolvedor ou um novo agente de IA começa a trabalhar em uma base de código, executar o AigisCode fornece uma visão geral estrutural instantânea. O relatório mostra onde estão os hotspots de dependência, quais módulos são mais acoplados e onde reside a dívida técnica conhecida. Isso é mais rápido e confiável do que ler documentação que pode estar desatualizada.</p>

<h2 id="the-future-of-ai-driven-maintenance">O Futuro da Manutenção Orientada por IA</h2>

<p>Estamos caminhando para um mundo onde a manutenção da base de código é amplamente automatizada. Agentes de IA monitorarão continuamente métricas de qualidade de código, detectarão degradação e aplicarão correções sem intervenção humana para problemas rotineiros. Desenvolvedores humanos se concentrarão em decisões arquiteturais, design de produto e orientação de alto nível que os agentes precisam para trabalhar efetivamente.</p>

<p>O AigisCode é projetado para esse futuro. Sua saída estruturada, comportamento orientado por políticas e separação clara entre análise determinística e classificação por IA o tornam naturalmente adequado para fluxos de trabalho orientados por agentes. A ferramenta fornece os olhos. O agente fornece as mãos. E o arquivo de política fornece o conhecimento institucional que garante que ambos trabalhem juntos efetivamente.</p>

<p>As equipes que integrarem essas ferramentas em seu fluxo de trabalho agora terão uma vantagem significativa. Não apenas código mais limpo, mas um sistema que fica mais inteligente ao longo do tempo à medida que a política acumula conhecimento específico do projeto e os agentes melhoram na interpretação e atuação sobre os resultados da análise. O futuro da qualidade de código não é um linter melhor. É um sistema inteligente onde análise, política e agentes autônomos trabalham juntos para manter as bases de código saudáveis em escala.</p>
`,
      ar: `

<p>لا يشبه سير عمل التطوير في عام 2026 ما كان لدينا قبل خمس سنوات. وكلاء البرمجة بالذكاء الاصطناعي، الأنظمة المستقلة التي يمكنها قراءة وفهم وتعديل واختبار الشيفرة، أصبحت الآن جزءاً منتظماً من هندسة البرمجيات. يمكن لوكلاء مثل Claude Code و GitHub Copilot Workspace و Codex تنفيذ الميزات من أوصاف المشكلات وإصلاح الأخطاء من سجلات الأخطاء وإعادة هيكلة الشيفرة بناءً على الأهداف المعمارية. لكن كل وكيل يشترك في حاجة مشتركة: معلومات منظمة وموثوقة حول قاعدة الشيفرة التي يعملون فيها.</p>

<p>هنا يتناسب AigisCode مع سير عمل وكيل الذكاء الاصطناعي. ليس كذكاء اصطناعي آخر يولد شيفرة، بل كطبقة تحليلية تمنح الوكلاء السياق الذي يحتاجونه لاتخاذ قرارات جيدة حول جودة الشيفرة والبنية المعمارية.</p>

<h2 id="the-agent-workflow">سير عمل التحليل-التحليل-الفرز-الإصلاح</h2>

<p>يتبع سير العمل الموصى به لوكيل ذكاء اصطناعي يستخدم AigisCode أربع مراحل، كل منها يبني على السابقة.</p>

<h3>المرحلة 1: التحليل</h3>

<p>يشغّل الوكيل <code>aigiscode analyze /path/to/project</code>. ينفذ هذا خط الأنابيب الكامل من ست مراحل: فهرسة الملفات المصدرية، وبناء رسم التبعيات البياني، وتشغيل الكواشف، وتطبيق قواعد الاستبعاد المحفوظة، وتشغيل مراجعة الذكاء الاصطناعي للتصنيف، وتوليد التقرير. المخرجات هي ملف JSON منظم في <code>.aigiscode/deterministic-analysis.json</code>.</p>

<p>هذه الخطوة حتمية باستثناء مرحلة مراجعة الذكاء الاصطناعي الاختيارية. يمكن للوكيل التشغيل مع <code>--skip-ai</code> لتحليل حتمي بالكامل، وهو مفيد في بيئات CI حيث تكون قابلية إعادة الإنتاج أهم من التصنيف الدقيق.</p>

<h3>المرحلة 2: التحليل</h3>

<p>يقرأ الوكيل تقرير JSON ويستخرج النتائج المنظمة. التقرير منظم في أقسام واضحة يمكن للوكيل التنقل فيها برمجياً.</p>

<p>يحتوي <code>graph_analysis.strong_circular_dependencies</code> على الدورات المعمارية التي تحتاج إلى إعادة هيكلة. تسرد كل دورة الوحدات المشاركة وحواف الاستيراد المحددة التي تشكل الحلقة. يحدد <code>graph_analysis.god_classes</code> الفئات ذات الاقتران والمسؤولية المفرطة. يبرز <code>graph_analysis.bottleneck_files</code> الملفات التي تقع على عدد كبير جداً من مسارات التبعية. يُفهرس <code>dead_code</code> الاستيرادات غير المستخدمة والأساليب غير المرجعية والفئات المهجورة والملفات اليتيمة. يسرد <code>hardwiring</code> السلاسل السحرية والقيم الحرفية المكررة وعناوين الشبكة المضمنة.</p>

<p>يتضمن كل اكتشاف مستوى ثقة (عالٍ، متوسط، منخفض) وتصنيف خطورة، مما يمنح الوكيل معلومات كافية لاتخاذ قرارات الفرز دون مدخلات بشرية.</p>

<h3>المرحلة 3: الفرز</h3>

<p>هنا يطبق الوكيل الحكم. ليست كل النتائج متساوية في الأهمية، وليست كلها إيجابيات حقيقية. يبدو سير عمل فرز الوكيل المصمم جيداً هكذا.</p>

<p>أولاً، رشّح حسب الثقة. ابدأ بالنتائج عالية الثقة. هذه لديها أدنى معدل إيجابيات كاذبة وتوفر الإشارة الأكثر موثوقية حول المشكلات الفعلية. يجب أخذ عينات من النتائج متوسطة الثقة والتحقق منها قبل التصرف عليها بالجملة. النتائج منخفضة الثقة معلوماتية.</p>

<p>ثانياً، حدد الأولويات حسب التأثير. تبعية دائرية بين وحدتين أساسيتين تستوردهما كل ميزة أكثر تأثيراً من تبعية دائرية بين نصين مساعدين. اكتشاف شيفرة ميتة في وحدة بها 50 تبعية واردة أهم من واحد في مساعد اختبار معزول.</p>

<p>ثالثاً، صنّف نوع الإصلاح. بعض النتائج تتطلب حذفاً بسيطاً (استيرادات غير مستخدمة، ملفات يتيمة). بعضها يتطلب تغييرات في الواجهة (تبعيات دائرية). بعضها يتطلب تحديثات تكوين (قيم مضمنة). يجب على الوكيل تجميع الإصلاحات المتشابهة معاً للتنفيذ الفعال.</p>

<h3>المرحلة 4: الإصلاح</h3>

<p>يطبق الوكيل الإصلاحات بناءً على نتائج الفرز. للإصلاحات البسيطة مثل إزالة الاستيرادات غير المستخدمة أو حذف الملفات اليتيمة، يمكن للوكيل التصرف بثقة عالية. للإصلاحات الهيكلية مثل كسر التبعيات الدائرية، يجب على الوكيل اقتراح خطة وتنفيذها وتشغيل مجموعة الاختبارات للتحقق من أن التغيير لا يكسر شيئاً.</p>

<p>بعد تطبيق الإصلاحات، يشغّل الوكيل <code>aigiscode analyze /path/to/project</code> لإعادة توليد التقرير من الفهرس الموجود. هذا أسرع من التحليل الكامل لأنه يتخطى مرحلة الفهرسة. يمكن للوكيل بعد ذلك مقارنة التقرير الجديد بخط الأساس للتحقق من أن النتائج قد حُلّت ولم يتم تقديم مشكلات جديدة.</p>

<h2 id="how-agents-consume-reports">كيف يستهلك الوكلاء الحقيقيون تقارير AigisCode</h2>

<p>لدى وكلاء الذكاء الاصطناعي المختلفين نقاط قوة مختلفة، والطريقة التي يستهلكون بها تقارير AigisCode تعكس تلك الاختلافات.</p>

<h3>Claude Code</h3>

<p>يتفوق وكلاء Claude Code في فهم السياق واتخاذ قرارات دقيقة. عند إعطائهم تقرير AigisCode، يمكن لوكيل Claude قراءة جميع النتائج وفهم العلاقات بينها وتطوير خطة إعادة هيكلة متماسكة تعالج مشكلات متعددة في وقت واحد. على سبيل المثال، إذا أظهر التقرير تبعية دائرية بين وحدتي <code>auth</code> و <code>users</code> وأيضاً شيفرة ميتة في كلتا الوحدتين، يمكن لـ Claude اقتراح إعادة هيكلة واحدة تكسر الدورة وتزيل الشيفرة الميتة في تغيير واحد متماسك.</p>

<p>تنسيق JSON المنظم مناسب بشكل خاص لنافذة سياق Claude. يمكن للوكيل تحميل التقرير بأكمله ومراجعة النتائج المتقاطعة مع الشيفرة المصدرية الفعلية وإنتاج إصلاحات تراعي السياق الكامل لكل مشكلة.</p>

<h3>وكلاء Codex</h3>

<p>وكلاء Codex فعالون في تنفيذ إصلاحات مستهدفة ضمن نطاق محدد. يعملون بشكل جيد عند إعطائهم نتيجة محددة من تقرير AigisCode وطلب إصلاحها. على سبيل المثال، عند إعطاء نتيجة شيفرة ميتة لأسلوب غير مستخدم محدد، يمكن لوكيل Codex تحديد جميع الشيفرة ذات الصلة والتحقق من أن الأسلوب غير مستخدم فعلاً وإنتاج حذف نظيف مع استيرادات محدثة.</p>

<p>توفر وحدة عمال AigisCode (<code>workers/codex.py</code>) نقاط تكامل مصممة خصيصاً لوكلاء نمط Codex الذين يعالجون النتائج واحدة تلو الأخرى.</p>

<h2 id="policy-driven-behavior">سلوك الوكيل المدفوع بالسياسة</h2>

<p>أحد مبادئ التصميم الرئيسية لـ AigisCode هو أن السلوك الخاص بالمشروع يجب أن يعيش في السياسة، وليس في الشيفرة. يمتد هذا المبدأ بشكل طبيعي إلى سير عمل وكلاء الذكاء الاصطناعي.</p>

<p>عندما يواجه وكيل إيجابية كاذبة، لا ينبغي أن يعدل المحلل. بدلاً من ذلك، يجب أن يضيف قاعدة استبعاد إلى <code>.aigiscode/rules.json</code>. عندما يحدد نمطاً من الإيجابيات الكاذبة (على سبيل المثال، جميع الأساليب في مجلد <code>Contracts/</code> تُعلّم كشيفرة ميتة لأنها منفذة عبر واجهات)، يجب أن يرمّز النمط في <code>.aigiscode/policy.json</code>.</p>

<p>لهذا النهج عدة مزايا. أولاً، يبقي تغييرات الوكيل قابلة للمراجعة. تغيير السياسة هو تعديل JSON صغير يمكن للمراجع البشري تقييمه بسرعة. ثانياً، يبقي التحليل قابلاً لإعادة الإنتاج. سيستفيد الوكلاء الآخرون والمطورون الآخرون الذين يشغلون AigisCode على نفس المشروع من معرفة السياسة المتراكمة. ثالثاً، يتبع مبدأ الحد الأدنى من الامتيازات. الوكيل الذي يحدث ملفات السياسة لا يمكنه كسر أداة التحليل نفسها بالخطأ.</p>

<p>يدعم ملف السياسة مجموعة غنية من خيارات التكوين. أسماء الاستيراد المستعارة (<code>js_import_aliases</code>) تخبر باني الرسم البياني كيفية حل أسماء المسارات المستعارة مثل <code>@/</code> في مشاريع TypeScript. أنماط نقاط الدخول (<code>orphan_entry_patterns</code>) تعلّم الملفات التي هي نقاط دخول شرعية حتى لو لم يستوردها شيء، مثل نصوص CLI أو تجهيزات الاختبار. أنماط الهجر (<code>abandoned_entry_patterns</code>) تخبر كاشف الشيفرة الميتة أي المجلدات تحتوي على تنفيذات واجهة لا ينبغي تعليمها.</p>

<h2 id="practical-integration-examples">أمثلة عملية للتكامل</h2>

<h3>تكامل خط أنابيب CI</h3>

<p>نمط التكامل الأكثر شيوعاً هو تشغيل AigisCode في CI وجعل وكيل يعالج النتائج. يشغّل خط أنابيب CI الأمر <code>aigiscode analyze .</code> على كل طلب سحب. إذا ظهرت نتائج جديدة مقارنة بخط الأساس على الفرع الرئيسي، يطلق خط الأنابيب وكيلاً لمراجعة النتائج وتصنيفها وإما إصلاحها تلقائياً أو ترك تعليقات مراجعة على طلب السحب.</p>

<pre><code># .github/workflows/code-quality.yml
- name: Run AigisCode Analysis
  run: aigiscode analyze .
- name: Compare with baseline
  run: diff .aigiscode/deterministic-analysis.json baseline-report.json
- name: Agent review (if new findings)
  run: agent review --report .aigiscode/deterministic-analysis.json</code></pre>

<h3>الصيانة المجدولة</h3>

<p>نمط آخر هو صيانة قاعدة الشيفرة المجدولة. يشغّل وكيل AigisCode أسبوعياً ويراجع التقرير الكامل وينشئ طلب سحب صيانة يعالج النتائج ذات الأولوية القصوى. يخلق هذا إيقاعاً ثابتاً للتحسين الهيكلي دون مطالبة المطورين بفرز المشكلات المعمارية يدوياً.</p>

<h3>تقارير التأهيل</h3>

<p>عندما يبدأ مطور جديد أو وكيل ذكاء اصطناعي جديد العمل على قاعدة شيفرة، يوفر تشغيل AigisCode نظرة عامة هيكلية فورية. يُظهر التقرير أين توجد نقاط اتصال التبعيات وأي الوحدات أكثر اقتراناً وأين تعيش الديون التقنية المعروفة. هذا أسرع وأكثر موثوقية من قراءة وثائق قد تكون قديمة.</p>

<h2 id="the-future-of-ai-driven-maintenance">مستقبل الصيانة المدفوعة بالذكاء الاصطناعي</h2>

<p>نحن نتجه نحو عالم تكون فيه صيانة قاعدة الشيفرة مؤتمتة إلى حد كبير. ستراقب وكلاء الذكاء الاصطناعي باستمرار مقاييس جودة الشيفرة وتكشف التدهور وتطبق الإصلاحات دون تدخل بشري للمشكلات الروتينية. سيركز المطورون البشريون على القرارات المعمارية وتصميم المنتج والتوجيه عالي المستوى الذي يحتاجه الوكلاء للعمل بفعالية.</p>

<p>صُمم AigisCode لهذا المستقبل. مخرجاته المنظمة وسلوكه المدفوع بالسياسة والفصل الواضح بين التحليل الحتمي وتصنيف الذكاء الاصطناعي يجعله مناسباً بشكل طبيعي لسير العمل المدفوع بالوكلاء. توفر الأداة العيون. يوفر الوكيل الأيدي. ويوفر ملف السياسة المعرفة المؤسسية التي تضمن أن كليهما يعملان معاً بفعالية.</p>

<p>الفرق التي تدمج هذه الأدوات في سير عملها الآن ستتمتع بميزة كبيرة. ليس فقط شيفرة أنظف، بل نظام يصبح أذكى بمرور الوقت حيث تراكم السياسة معرفة خاصة بالمشروع ويصبح الوكلاء أفضل في تفسير نتائج التحليل والتصرف بناءً عليها. مستقبل جودة الشيفرة ليس أداة فحص أفضل. إنه نظام ذكي حيث يعمل التحليل والسياسة والوكلاء المستقلون معاً للحفاظ على صحة قواعد الشيفرة على نطاق واسع.</p>
`,
      pl: `<h2 id="ai-agents">Agenci AI i AigisCode</h2>
<p>Agenci AI potrzebują ustrukturyzowanych danych o kondycji kodu. AigisCode generuje czytelne maszynowo raporty JSON dla autonomicznego naprawiania problemów.</p>`,
      bn: `
<p>২০২৬-এর ডেভেলপমেন্ট ওয়ার্কফ্লো পাঁচ বছর আগের থেকে সম্পূর্ণ ভিন্ন দেখায়। AI কোডিং এজেন্ট, স্বায়ত্তশাসিত সিস্টেম যা কোড পড়তে, বুঝতে, পরিবর্তন করতে এবং টেস্ট করতে পারে, এখন সফটওয়্যার ইঞ্জিনিয়ারিংয়ের একটি নিয়মিত অংশ। Claude Code, GitHub Copilot Workspace, এবং Codex-এর মতো এজেন্টরা ফিচার ইম্প্লিমেন্ট, বাগ ফিক্স এবং কোড রিফ্যাক্টর করতে পারে। কিন্তু প্রতিটি এজেন্টের কোডবেস সম্পর্কে স্ট্রাক্চার্ড, নির্ভরযোগ্য তথ্য প্রয়োজন।</p>

<p>এখানেই AigisCode AI এজেন্ট ওয়ার্কফ্লোতে মানানসই হয়। বিশ্লেষণাত্মক স্তর হিসেবে যা এজেন্টদের কোড কোয়ালিটি এবং আর্কিটেকচার সম্পর্কে ভালো সিদ্ধান্ত নেওয়ার জন্য প্রয়োজনীয় প্রসঙ্গ দেয়।</p>

<h2 id="the-agent-workflow">Analyze-Parse-Triage-Fix ওয়ার্কফ্লো</h2>

<p>এজেন্ট <code>aigiscode analyze /path/to/project</code> চালায়, JSON রিপোর্ট পার্স করে, কনফিডেন্স এবং প্রভাব দ্বারা ফলাফল ট্রায়াজ করে, এবং তারপর ফিক্স প্রয়োগ করে। <code>graph_analysis.strong_circular_dependencies</code>-তে আর্কিটেকচারাল সাইকেল, <code>dead_code</code>-এ অব্যবহৃত কোড, এবং <code>hardwiring</code>-এ হার্ডকোডেড ভ্যালু থাকে।</p>

<h2 id="how-agents-consume-reports">এজেন্টরা কিভাবে রিপোর্ট ব্যবহার করে</h2>

<p>Claude Code এজেন্টরা সুসংগত রিফ্যাক্টরিং পরিকল্পনা তৈরি করতে পারে। Codex এজেন্টরা লক্ষ্যবস্তু ফিক্স এক্সিকিউট করতে কার্যকর। AigisCode workers মডিউল (<code>workers/codex.py</code>) Codex-স্টাইল এজেন্টদের জন্য ইন্টিগ্রেশন পয়েন্ট প্রদান করে।</p>

<h2 id="policy-driven-behavior">পলিসি-চালিত এজেন্ট আচরণ</h2>

<p>প্রজেক্ট-নির্দিষ্ট আচরণ পলিসিতে থাকে। False positive-এর জন্য <code>.aigiscode/rules.json</code>-এ এক্সক্লুশন নিয়ম যোগ করুন। প্যাটার্নের জন্য <code>.aigiscode/policy.json</code>-এ এনকোড করুন।</p>

<h2 id="practical-integration-examples">ব্যবহারিক ইন্টিগ্রেশন</h2>

<p>CI পাইপলাইনে প্রতিটি পুল রিকোয়েস্টে <code>aigiscode analyze .</code> চালান। সাপ্তাহিক রক্ষণাবেক্ষণ রিপোর্ট চালান। অনবোর্ডিংয়ে তাৎক্ষণিক স্ট্রাক্চারাল ওভারভিউ পান।</p>

<pre><code># .github/workflows/code-quality.yml
- name: Run AigisCode Analysis
  run: aigiscode analyze .
- name: Compare with baseline
  run: diff .aigiscode/deterministic-analysis.json baseline-report.json
- name: Agent review (if new findings)
  run: agent review --report .aigiscode/deterministic-analysis.json</code></pre>

<h2 id="the-future-of-ai-driven-maintenance">AI-চালিত রক্ষণাবেক্ষণের ভবিষ্যৎ</h2>

<p>AigisCode এজেন্ট-চালিত ওয়ার্কফ্লোর জন্য ডিজাইন করা হয়েছে। টুলটি চোখ প্রদান করে। এজেন্ট হাত প্রদান করে। পলিসি ফাইল প্রাতিষ্ঠানিক জ্ঞান প্রদান করে। কোড কোয়ালিটির ভবিষ্যৎ একটি বুদ্ধিমান সিস্টেম যেখানে বিশ্লেষণ, পলিসি এবং স্বায়ত্তশাসিত এজেন্ট একসাথে কাজ করে কোডবেসকে স্কেলে স্বাস্থ্যকর রাখতে।</p>
`,
    },
  },
];

/* -------------------------------------------------------------------------- */
/*  Helpers                                                                   */
/* -------------------------------------------------------------------------- */

/** Get a single post by slug */
export function getBlogPost(slug: string): BlogPost | undefined {
  return blogPosts.find((p) => p.slug === slug);
}

/** Get all unique tags from published posts */
export function getAllTags(): string[] {
  const set = new Set<string>();
  for (const post of blogPosts) {
    for (const tag of post.tags) {
      set.add(tag);
    }
  }
  return Array.from(set).sort();
}

/** Get posts that share at least one tag with the given post */
export function getRelatedPosts(post: BlogPost): BlogPost[] {
  return blogPosts.filter(
    (p) => p.slug !== post.slug && post.relatedSlugs.includes(p.slug),
  );
}
