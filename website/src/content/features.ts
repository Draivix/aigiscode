/* -------------------------------------------------------------------------- */
/*  Feature Data — AigisCode Marketing Website                               */
/* -------------------------------------------------------------------------- */

export interface Feature {
  slug: string;
  icon: string;
  category: string;
  title: Record<string, string>;
  shortDescription: Record<string, string>;
  metaDescription: Record<string, string>;
  content: Record<string, string>;
  capabilities: string[];
  codeExample?: string;
  relatedSlugs: string[];
}

export const features: Feature[] = [
  /* ---- 1. Circular Dependencies ---- */
  {
    slug: 'circular-dependencies',
    icon: 'ArrowsClockwise',
    category: 'Architecture',
    title: {
      en: 'Circular Dependency Detection',
      cs: 'Detekce cyklických závislostí',
      fr: 'Détection des dépendances circulaires',
      es: 'Detección de dependencias circulares',
      zh: '循环依赖检测',
      hi: 'चक्रीय निर्भरता पहचान',
      pt: 'Detecção de dependências circulares',
      ar: 'اكتشاف التبعيات الدائرية',
      pl: 'Wykrywanie cyklicznych zaleznosci',
      bn: 'সার্কুলার ডিপেন্ডেন্সি ডিটেকশন',
    },
    shortDescription: {
      en: 'Detect architectural cycles between modules that create tight coupling and make refactoring dangerous. AigisCode distinguishes strong architectural cycles from benign runtime load-order dependencies using Tarjan SCC analysis.',
      cs: 'Detekujte architektonické cykly mezi moduly, které vytvářejí těsnou vazbu a činí refaktoring nebezpečným. AigisCode rozlišuje silné architektonické cykly od neškodných běhových závislostí pomocí Tarjanovy SCC analýzy.',
      fr: 'Détectez les cycles architecturaux entre modules qui créent un couplage fort et rendent le refactoring dangereux. AigisCode distingue les cycles architecturaux forts des dépendances bénignes d\'ordre de chargement grâce à l\'analyse Tarjan SCC.',
      es: 'Detecta ciclos arquitectónicos entre módulos que generan un acoplamiento fuerte y hacen peligrosa la refactorización. AigisCode distingue los ciclos arquitectónicos fuertes de las dependencias benignas de orden de carga mediante el análisis Tarjan SCC.',
      zh: '检测模块之间的架构循环依赖，这些循环会造成紧耦合并使重构变得危险。AigisCode 使用 Tarjan SCC 分析区分强架构循环和良性的运行时加载顺序依赖。',
      hi: 'मॉड्यूल के बीच आर्किटेक्चरल चक्रों का पता लगाएं जो टाइट कपलिंग बनाते हैं और रिफैक्टरिंग को खतरनाक बनाते हैं। AigisCode Tarjan SCC विश्लेषण का उपयोग करके मजबूत आर्किटेक्चरल चक्रों को सामान्य रनटाइम लोड-ऑर्डर निर्भरताओं से अलग करता है।',
      pt: 'Detecte ciclos arquiteturais entre módulos que criam acoplamento forte e tornam a refatoração perigosa. AigisCode distingue ciclos arquiteturais fortes de dependências benignas de ordem de carregamento usando análise Tarjan SCC.',
      ar: 'اكتشف الدورات المعمارية بين الوحدات التي تخلق اقتراناً محكماً وتجعل إعادة البناء خطيرة. يميّز AigisCode الدورات المعمارية القوية عن تبعيات ترتيب التحميل الحميدة باستخدام تحليل Tarjan SCC.',
      pl: 'Wykrywaj architektoniczne cykle miedzy modulami, ktore tworza scisle powiazania i utrudniaja refaktoryzacje. AigisCode rozroznia silne cykle architektoniczne od nieszkodliwych zaleznosci kolejnosci ladowania za pomoca analizy Tarjan SCC.',
      bn: 'মডিউলগুলোর মধ্যে আর্কিটেকচারাল চক্র শনাক্ত করুন যা টাইট কাপলিং তৈরি করে এবং রিফ্যাক্টরিং ঝুঁকিপূর্ণ করে। AigisCode Tarjan SCC বিশ্লেষণ ব্যবহার করে শক্তিশালী আর্কিটেকচারাল চক্রকে সাধারণ রানটাইম লোড-অর্ডার ডিপেন্ডেন্সি থেকে আলাদা করে।',
    },
    metaDescription: {
      en: 'Detect circular dependencies in PHP, Python, TypeScript, and JavaScript codebases. AigisCode uses petgraph graph analysis and Tarjan SCC to find strong architectural cycles that single-file linters miss.',
      cs: 'Detekujte cyklické závislosti v kódových základnách PHP, Python, TypeScript a JavaScript. AigisCode používá grafovou analýzu petgraph a Tarjan SCC k nalezení silných architektonických cyklů, které jednoduchá analýza souborů přehlédne.',
      fr: 'Détectez les dépendances circulaires dans les bases de code PHP, Python, TypeScript et JavaScript. AigisCode utilise l\'analyse de graphes petgraph et Tarjan SCC pour trouver les cycles architecturaux forts que les linters mono-fichier ne détectent pas.',
      es: 'Detecta dependencias circulares en bases de código PHP, Python, TypeScript y JavaScript. AigisCode utiliza análisis de grafos petgraph y Tarjan SCC para encontrar ciclos arquitectónicos fuertes que los linters de archivo único no detectan.',
      zh: '检测 PHP、Python、TypeScript 和 JavaScript 代码库中的循环依赖。AigisCode 使用 petgraph 图分析和 Tarjan SCC 发现单文件 linter 无法检测到的强架构循环。',
      hi: 'PHP, Python, TypeScript और JavaScript कोडबेस में सर्कुलर डिपेंडेंसी का पता लगाएं। AigisCode petgraph ग्राफ विश्लेषण और Tarjan SCC का उपयोग करके उन मजबूत आर्किटेक्चरल चक्रों को खोजता है जो सिंगल-फाइल लिंटर नहीं पकड़ पाते।',
      pt: 'Detecte dependências circulares em bases de código PHP, Python, TypeScript e JavaScript. AigisCode usa análise de grafos petgraph e Tarjan SCC para encontrar ciclos arquiteturais fortes que linters de arquivo único não detectam.',
      ar: 'اكتشف التبعيات الدائرية في قواعد شيفرة PHP وPython وTypeScript وJavaScript. يستخدم AigisCode تحليل رسوم petgraph البيانية وTarjan SCC للعثور على الدورات المعمارية القوية التي تفوتها أدوات فحص الملف الواحد.',
      pl: 'Wykrywaj cykliczne zaleznosci w bazach kodu PHP, Python, TypeScript i JavaScript. AigisCode znajduje silne cykle architektoniczne, ktore jednorazowe lintery pomijaja.',
      bn: 'PHP, Python, TypeScript এবং JavaScript কোডবেসে সার্কুলার ডিপেন্ডেন্সি শনাক্ত করুন। AigisCode petgraph গ্রাফ বিশ্লেষণ এবং Tarjan SCC ব্যবহার করে সিঙ্গেল-ফাইল লিন্টার মিস করে এমন শক্তিশালী আর্কিটেকচারাল চক্র খুঁজে বের করে।',
    },
    content: {
      en: `<h2 id="what-are-circular-dependencies">What Are Circular Dependencies?</h2>
<p>Circular dependencies occur when two or more modules depend on each other, directly or indirectly. Module A imports Module B, which imports Module C, which imports Module A — creating a cycle. These cycles are among the most insidious architectural problems in modern codebases because they silently erode modularity, make testing painful, and can lead to unpredictable initialization behavior.</p>
<p>Most single-file linters cannot detect circular dependencies because the problem only becomes visible when you analyze the <strong>entire dependency graph</strong> of a project. A file in isolation looks fine. It is only when you trace the import chain across dozens of files that cycles emerge.</p>

<h2 id="strong-vs-runtime-cycles">Strong Cycles vs. Runtime Cycles</h2>
<p>AigisCode makes a critical distinction between two types of circular dependencies:</p>
<ul>
<li><strong>Strong architectural cycles</strong> — These are cycles in the static import graph where modules are structurally coupled. Every module in the cycle is reachable from every other module. Refactoring one module forces changes to all others. These are high-priority findings reported under <code>graph_analysis.strong_circular_dependencies</code>.</li>
<li><strong>Runtime load-order cycles</strong> — These occur when modules have circular imports that are resolved at runtime through lazy loading, deferred access, or dynamic imports. They are lower priority but still tracked under <code>graph_analysis.circular_dependencies</code>.</li>
</ul>
<p>This distinction eliminates false positives. A Django project might have models that reference each other through string-based foreign keys — technically a cycle in the import graph, but not a structural coupling problem. AigisCode's graph analyzer uses strongly connected component (SCC) analysis via Tarjan's algorithm to find only the cycles that genuinely hurt your codebase.</p>

<h2 id="how-detection-works">How Detection Works</h2>
<p>The detection process runs through AigisCode's six-stage pipeline:</p>
<ol>
<li><strong>Index</strong> — AigisCode scans the repository, parses supported languages with native Rust adapters, and extracts symbols, references, and evidence into a canonical semantic graph.</li>
<li><strong>Graph</strong> — A layered dependency graph is built using petgraph. Structural, runtime, and framework edges are separated so architecture queries can stay low-noise while evidence remains preserved.</li>
<li><strong>Detect</strong> — The graph analyzer runs Tarjan's algorithm to find strongly connected components. Components with more than one node represent architectural cycles. Both direct (A-B-A) and transitive (A-B-C-D-A) cycles are detected.</li>
</ol>
<p>Each finding includes the complete cycle chain — every file that participates in the cycle — along with a confidence score. The cycle chain is essential for deciding where to break the loop.</p>

<h2 id="reading-the-report">Reading the Report</h2>
<p>The JSON report separates strong and runtime cycles:</p>
<pre><code>// .aigiscode/deterministic-analysis.json
{
  "graph_analysis": {
    "strong_circular_dependencies": [
      {
        "cycle": ["src/auth/service.ts", "src/users/repository.ts", "src/auth/middleware.ts"],
        "edge_count": 3,
        "confidence": "high"
      }
    ],
    "circular_dependencies": [
      // All cycles including runtime/load-order
    ]
  }
}</code></pre>
<p>Start with strong cycles. They are the highest-value fixes because breaking them directly improves modularity, testability, and build performance.</p>

<h2 id="breaking-cycles">Strategies for Breaking Cycles</h2>
<p>Once AigisCode identifies your cycles, common resolution strategies include:</p>
<ul>
<li><strong>Extract shared types</strong> — Move interfaces and type definitions into a dedicated <code>types/</code> or <code>contracts/</code> directory that has no outbound dependencies.</li>
<li><strong>Dependency inversion</strong> — Instead of Module A importing Module B directly, define an interface in Module A that Module B implements. This inverts the dependency direction.</li>
<li><strong>Lazy imports</strong> — In Python, move imports inside functions. In TypeScript, use dynamic <code>import()</code>. This breaks the static cycle while preserving runtime access.</li>
<li><strong>Mediator pattern</strong> — Introduce a third module that both A and B depend on, eliminating the direct cycle.</li>
</ul>`,
      cs: `<h2 id="what-are-circular-dependencies">Co jsou cyklické závislosti?</h2>
<p>Cyklické závislosti vznikají, když dva nebo více modulů závisí na sobě navzájem, přímo nebo nepřímo. Modul A importuje Modul B, který importuje Modul C, který importuje Modul A — čímž vzniká cyklus. Tyto cykly patří k nejzákeřnějším architektonickým problémům v moderních kódových základnách, protože tiše narušují modularitu, ztěžují testování a mohou vést k nepředvídatelnému chování při inicializaci.</p>

<h2 id="strong-vs-runtime-cycles">Silné cykly vs. běhové cykly</h2>
<p>AigisCode rozlišuje dva typy cyklických závislostí:</p>
<ul>
<li><strong>Silné architektonické cykly</strong> — Cykly ve statickém importním grafu, kde jsou moduly strukturálně propojeny. Refaktoring jednoho modulu si vynutí změny všech ostatních. Tyto nálezy jsou hlášeny pod <code>graph_analysis.strong_circular_dependencies</code>.</li>
<li><strong>Běhové cykly pořadí načítání</strong> — Vznikají, když moduly mají cyklické importy řešené za běhu pomocí lazy loadingu nebo dynamických importů. Mají nižší prioritu, ale jsou sledovány pod <code>graph_analysis.circular_dependencies</code>.</li>
</ul>
<p>Toto rozlišení eliminuje falešné pozitivy. AigisCode používá analýzu silně propojených komponent (SCC) pomocí Tarjanova algoritmu k nalezení pouze cyklů, které skutečně poškozují vaši kódovou základnu.</p>

<h2 id="how-detection-works">Jak detekce funguje</h2>
<p>Proces detekce probíhá ve třech fázích:</p>
<ol>
<li><strong>Indexace</strong> — AigisCode parsuje každý soubor pomocí tree-sitter (pro PHP, TypeScript, JavaScript, Vue) nebo tree-sitter-python a extrahuje všechny importní příkazy.</li>
<li><strong>Graf</strong> — Pomocí petgraph se vytvoří graf závislostí na úrovni souborů. Aliasy importů (jako <code>@/</code> mapující na <code>src/</code>) jsou řešeny pomocí vaší konfigurace <code>policy.json</code>.</li>
<li><strong>Detekce</strong> — Analyzátor grafu spustí Tarjanův algoritmus k nalezení silně propojených komponent. Komponenty s více než jedním uzlem představují architektonické cykly.</li>
</ol>

<h2 id="breaking-cycles">Strategie pro rozbití cyklů</h2>
<ul>
<li><strong>Extrahujte sdílené typy</strong> — Přesuňte rozhraní a definice typů do dedikovaného adresáře <code>types/</code> nebo <code>contracts/</code>.</li>
<li><strong>Inverze závislostí</strong> — Místo přímého importu definujte rozhraní, které implementuje závislý modul.</li>
<li><strong>Lazy importy</strong> — V Pythonu přesuňte importy dovnitř funkcí. V TypeScriptu použijte dynamický <code>import()</code>.</li>
<li><strong>Mediátor pattern</strong> — Zaveďte třetí modul, na kterém oba závisí, čímž eliminujete přímý cyklus.</li>
</ul>`,
      fr: `<h2 id="what-are-circular-dependencies">Que sont les dépendances circulaires ?</h2>
<p>Les dépendances circulaires surviennent lorsque deux modules ou plus dépendent les uns des autres, directement ou indirectement. Le Module A importe le Module B, qui importe le Module C, qui importe le Module A — créant un cycle. Ces cycles comptent parmi les problèmes architecturaux les plus insidieux dans les bases de code modernes car ils érodent silencieusement la modularité et rendent les tests difficiles.</p>

<h2 id="strong-vs-runtime-cycles">Cycles forts vs. cycles d'exécution</h2>
<p>AigisCode distingue deux types de dépendances circulaires :</p>
<ul>
<li><strong>Cycles architecturaux forts</strong> — Cycles dans le graphe d'import statique où les modules sont structurellement couplés. Le refactoring d'un module force des changements dans tous les autres. Reportés sous <code>graph_analysis.strong_circular_dependencies</code>.</li>
<li><strong>Cycles d'ordre de chargement à l'exécution</strong> — Se produisent lorsque les modules ont des imports circulaires résolus à l'exécution via le chargement paresseux ou les imports dynamiques. Priorité inférieure, suivis sous <code>graph_analysis.circular_dependencies</code>.</li>
</ul>
<p>Cette distinction élimine les faux positifs. AigisCode utilise l'analyse des composants fortement connectés (SCC) via l'algorithme de Tarjan pour trouver uniquement les cycles qui nuisent réellement à votre code.</p>

<h2 id="how-detection-works">Comment fonctionne la détection</h2>
<p>Le processus de détection se déroule en trois étapes :</p>
<ol>
<li><strong>Indexation</strong> — AigisCode analyse chaque fichier avec tree-sitter (PHP, TypeScript, JavaScript, Vue) ou tree-sitter-python, extrayant toutes les instructions d'import.</li>
<li><strong>Graphe</strong> — Un graphe de dépendances au niveau fichier est construit avec petgraph. Les alias d'import (comme <code>@/</code> vers <code>src/</code>) sont résolus via votre <code>policy.json</code>.</li>
<li><strong>Détection</strong> — L'analyseur de graphe exécute l'algorithme de Tarjan pour trouver les composants fortement connectés.</li>
</ol>

<h2 id="breaking-cycles">Stratégies pour briser les cycles</h2>
<ul>
<li><strong>Extraire les types partagés</strong> — Déplacez les interfaces dans un répertoire <code>types/</code> ou <code>contracts/</code> dédié.</li>
<li><strong>Inversion de dépendance</strong> — Définissez une interface que le module dépendant implémente au lieu d'importer directement.</li>
<li><strong>Imports paresseux</strong> — En Python, déplacez les imports dans les fonctions. En TypeScript, utilisez <code>import()</code> dynamique.</li>
<li><strong>Pattern médiateur</strong> — Introduisez un troisième module dont les deux dépendent, éliminant le cycle direct.</li>
</ul>`,
      es: `<h2 id="what-are-circular-dependencies">Que son las dependencias circulares?</h2>
<p>Las dependencias circulares ocurren cuando dos o mas modulos dependen entre si, directa o indirectamente. El Modulo A importa el Modulo B, que importa el Modulo C, que importa el Modulo A — creando un ciclo. Estos ciclos son de los problemas arquitectonicos mas insidiosos en las bases de codigo modernas porque erosionan silenciosamente la modularidad y hacen las pruebas dificiles.</p>

<h2 id="strong-vs-runtime-cycles">Ciclos fuertes vs. ciclos de ejecucion</h2>
<p>AigisCode distingue dos tipos de dependencias circulares:</p>
<ul>
<li><strong>Ciclos arquitectonicos fuertes</strong> — Ciclos en el grafo de importacion estatico donde los modulos estan estructuralmente acoplados. Refactorizar un modulo fuerza cambios en todos los demas. Reportados bajo <code>graph_analysis.strong_circular_dependencies</code>.</li>
<li><strong>Ciclos de orden de carga en tiempo de ejecucion</strong> — Ocurren cuando los modulos tienen importaciones circulares resueltas en tiempo de ejecucion mediante carga diferida o importaciones dinamicas. Menor prioridad, rastreados bajo <code>graph_analysis.circular_dependencies</code>.</li>
</ul>
<p>Esta distincion elimina falsos positivos. AigisCode usa analisis de componentes fuertemente conectados (SCC) mediante el algoritmo de Tarjan para encontrar solo los ciclos que realmente danan su codigo.</p>

<h2 id="how-detection-works">Como funciona la deteccion</h2>
<p>El proceso de deteccion se ejecuta en tres etapas:</p>
<ol>
<li><strong>Indexacion</strong> — AigisCode analiza cada archivo con tree-sitter (PHP, TypeScript, JavaScript, Vue) o tree-sitter-python, extrayendo todas las declaraciones de importacion.</li>
<li><strong>Grafo</strong> — Se construye un grafo de dependencias a nivel de archivo con petgraph. Los alias de importacion (como <code>@/</code> a <code>src/</code>) se resuelven via <code>policy.json</code>.</li>
<li><strong>Deteccion</strong> — El analizador de grafos ejecuta el algoritmo de Tarjan para encontrar componentes fuertemente conectados.</li>
</ol>

<h2 id="breaking-cycles">Estrategias para romper ciclos</h2>
<ul>
<li><strong>Extraer tipos compartidos</strong> — Mueva las interfaces a un directorio dedicado <code>types/</code> o <code>contracts/</code>.</li>
<li><strong>Inversion de dependencia</strong> — Defina una interfaz que el modulo dependiente implemente en lugar de importar directamente.</li>
<li><strong>Importaciones diferidas</strong> — En Python, mueva los imports dentro de funciones. En TypeScript, use <code>import()</code> dinamico.</li>
<li><strong>Patron mediador</strong> — Introduzca un tercer modulo del que ambos dependan, eliminando el ciclo directo.</li>
</ul>`,
      zh: `<h2 id="what-are-circular-dependencies">什么是循环依赖？</h2>
<p>循环依赖发生在两个或更多模块直接或间接地相互依赖时。模块A导入模块B，模块B导入模块C，模块C又导入模块A——形成循环。这些循环是现代代码库中最隐蔽的架构问题之一，因为它们会悄然侵蚀模块化，使测试变得困难，并可能导致不可预测的初始化行为。</p>

<h2 id="strong-vs-runtime-cycles">强循环 vs. 运行时循环</h2>
<p>AigisCode区分两种类型的循环依赖：</p>
<ul>
<li><strong>强架构循环</strong> — 静态导入图中模块结构性耦合的循环。重构一个模块会迫使所有其他模块也必须更改。在 <code>graph_analysis.strong_circular_dependencies</code> 下报告。</li>
<li><strong>运行时加载顺序循环</strong> — 当模块通过懒加载或动态导入在运行时解析循环导入时发生。优先级较低，在 <code>graph_analysis.circular_dependencies</code> 下跟踪。</li>
</ul>
<p>这种区分消除了误报。AigisCode使用Tarjan算法的强连通分量（SCC）分析，仅找到真正损害代码库的循环。</p>

<h2 id="how-detection-works">检测原理</h2>
<p>检测过程分三个阶段：</p>
<ol>
<li><strong>索引</strong> — AigisCode使用tree-sitter（PHP、TypeScript、JavaScript、Vue）或tree-sitter-python解析每个文件，提取所有导入语句。</li>
<li><strong>图构建</strong> — 使用petgraph构建文件级依赖图。导入别名（如 <code>@/</code> 映射到 <code>src/</code>）通过 <code>policy.json</code> 配置解析。</li>
<li><strong>检测</strong> — 图分析器运行Tarjan算法查找强连通分量。包含多个节点的分量代表架构循环。</li>
</ol>

<h2 id="breaking-cycles">打破循环的策略</h2>
<ul>
<li><strong>提取共享类型</strong> — 将接口和类型定义移至专用的 <code>types/</code> 或 <code>contracts/</code> 目录。</li>
<li><strong>依赖倒置</strong> — 定义一个接口让依赖模块实现，而不是直接导入。</li>
<li><strong>延迟导入</strong> — 在Python中将导入移至函数内部。在TypeScript中使用动态 <code>import()</code>。</li>
<li><strong>中介者模式</strong> — 引入第三个模块，让两者都依赖它，从而消除直接循环。</li>
</ul>`,
      hi: `<h2 id="what-are-circular-dependencies">सर्कुलर डिपेंडेंसी क्या हैं?</h2>
<p>सर्कुलर डिपेंडेंसी तब होती है जब दो या अधिक मॉड्यूल प्रत्यक्ष या अप्रत्यक्ष रूप से एक-दूसरे पर निर्भर करते हैं। मॉड्यूल A, मॉड्यूल B को इम्पोर्ट करता है, जो मॉड्यूल C को इम्पोर्ट करता है, जो मॉड्यूल A को इम्पोर्ट करता है — एक चक्र बनाते हुए। ये चक्र आधुनिक कोडबेस में सबसे कपटी आर्किटेक्चरल समस्याओं में से हैं क्योंकि ये चुपचाप मॉड्यूलैरिटी को नष्ट करते हैं और टेस्टिंग को कठिन बनाते हैं।</p>

<h2 id="strong-vs-runtime-cycles">स्ट्रॉन्ग साइकल vs. रनटाइम साइकल</h2>
<p>AigisCode दो प्रकार की सर्कुलर डिपेंडेंसी में अंतर करता है:</p>
<ul>
<li><strong>स्ट्रॉन्ग आर्किटेक्चरल साइकल</strong> — स्टैटिक इम्पोर्ट ग्राफ में साइकल जहां मॉड्यूल संरचनात्मक रूप से जुड़े होते हैं। एक मॉड्यूल का रिफैक्टरिंग सभी अन्य में बदलाव को मजबूर करता है। <code>graph_analysis.strong_circular_dependencies</code> के तहत रिपोर्ट किए जाते हैं।</li>
<li><strong>रनटाइम लोड-ऑर्डर साइकल</strong> — जब मॉड्यूल में lazy loading या डायनामिक इम्पोर्ट के माध्यम से रनटाइम पर हल होने वाले सर्कुलर इम्पोर्ट होते हैं। कम प्राथमिकता, <code>graph_analysis.circular_dependencies</code> के तहत ट्रैक किए जाते हैं।</li>
</ul>
<p>यह अंतर फॉल्स पॉजिटिव को समाप्त करता है। AigisCode केवल उन साइकल को खोजने के लिए Tarjan एल्गोरिथम का SCC विश्लेषण उपयोग करता है जो वास्तव में आपके कोडबेस को नुकसान पहुंचाते हैं।</p>

<h2 id="how-detection-works">डिटेक्शन कैसे काम करता है</h2>
<ol>
<li><strong>इंडेक्सिंग</strong> — AigisCode tree-sitter (PHP, TypeScript, JavaScript, Vue) या tree-sitter-python का उपयोग करके हर फ़ाइल को पार्स करता है।</li>
<li><strong>ग्राफ</strong> — petgraph का उपयोग करके फ़ाइल-स्तरीय डिपेंडेंसी ग्राफ बनाया जाता है। इम्पोर्ट एलियास <code>policy.json</code> कॉन्फ़िगरेशन से हल किए जाते हैं।</li>
<li><strong>डिटेक्शन</strong> — ग्राफ एनालाइज़र Tarjan एल्गोरिथम चलाकर स्ट्रॉन्गली कनेक्टेड कंपोनेंट्स ढूंढता है।</li>
</ol>

<h2 id="breaking-cycles">साइकल तोड़ने की रणनीतियां</h2>
<ul>
<li><strong>शेयर्ड टाइप्स एक्सट्रैक्ट करें</strong> — इंटरफेस और टाइप डेफिनिशन को एक समर्पित <code>types/</code> या <code>contracts/</code> डायरेक्टरी में ले जाएं।</li>
<li><strong>डिपेंडेंसी इनवर्शन</strong> — सीधे इम्पोर्ट करने के बजाय एक इंटरफेस परिभाषित करें जिसे निर्भर मॉड्यूल इम्प्लीमेंट करे।</li>
<li><strong>लेज़ी इम्पोर्ट्स</strong> — Python में इम्पोर्ट को फंक्शन के अंदर ले जाएं। TypeScript में डायनामिक <code>import()</code> का उपयोग करें।</li>
<li><strong>मीडिएटर पैटर्न</strong> — एक तीसरा मॉड्यूल पेश करें जिस पर दोनों निर्भर हों, सीधे चक्र को समाप्त करते हुए।</li>
</ul>`,
      pt: `<h2 id="what-are-circular-dependencies">O que sao dependencias circulares?</h2>
<p>Dependencias circulares ocorrem quando dois ou mais modulos dependem uns dos outros, direta ou indiretamente. O Modulo A importa o Modulo B, que importa o Modulo C, que importa o Modulo A — criando um ciclo. Esses ciclos estao entre os problemas arquiteturais mais insidiosos em bases de codigo modernas porque corroem silenciosamente a modularidade e tornam os testes dificeis.</p>

<h2 id="strong-vs-runtime-cycles">Ciclos fortes vs. ciclos de execucao</h2>
<p>AigisCode distingue dois tipos de dependencias circulares:</p>
<ul>
<li><strong>Ciclos arquiteturais fortes</strong> — Ciclos no grafo de importacao estatico onde os modulos estao estruturalmente acoplados. Refatorar um modulo forca mudancas em todos os outros. Reportados sob <code>graph_analysis.strong_circular_dependencies</code>.</li>
<li><strong>Ciclos de ordem de carregamento em tempo de execucao</strong> — Ocorrem quando modulos tem importacoes circulares resolvidas em tempo de execucao via carregamento preguicoso ou importacoes dinamicas. Menor prioridade, rastreados sob <code>graph_analysis.circular_dependencies</code>.</li>
</ul>
<p>Esta distincao elimina falsos positivos. AigisCode usa analise de componentes fortemente conectados (SCC) via algoritmo de Tarjan para encontrar apenas os ciclos que realmente prejudicam seu codigo.</p>

<h2 id="how-detection-works">Como a deteccao funciona</h2>
<p>O processo de deteccao ocorre em tres etapas:</p>
<ol>
<li><strong>Indexacao</strong> — AigisCode analisa cada arquivo com tree-sitter (PHP, TypeScript, JavaScript, Vue) ou tree-sitter-python, extraindo todas as declaracoes de importacao.</li>
<li><strong>Grafo</strong> — Um grafo de dependencias ao nivel de arquivo e construido com petgraph. Alias de importacao (como <code>@/</code> para <code>src/</code>) sao resolvidos via <code>policy.json</code>.</li>
<li><strong>Deteccao</strong> — O analisador de grafo executa o algoritmo de Tarjan para encontrar componentes fortemente conectados.</li>
</ol>

<h2 id="breaking-cycles">Estrategias para quebrar ciclos</h2>
<ul>
<li><strong>Extrair tipos compartilhados</strong> — Mova interfaces para um diretorio dedicado <code>types/</code> ou <code>contracts/</code>.</li>
<li><strong>Inversao de dependencia</strong> — Defina uma interface que o modulo dependente implementa em vez de importar diretamente.</li>
<li><strong>Importacoes preguicosas</strong> — Em Python, mova imports para dentro de funcoes. Em TypeScript, use <code>import()</code> dinamico.</li>
<li><strong>Padrao mediador</strong> — Introduza um terceiro modulo do qual ambos dependam, eliminando o ciclo direto.</li>
</ul>`,
      ar: `<h2 id="what-are-circular-dependencies">ما هي التبعيات الدائرية؟</h2>
<p>تحدث التبعيات الدائرية عندما تعتمد وحدتان أو أكثر على بعضها البعض بشكل مباشر أو غير مباشر. تستورد الوحدة A من الوحدة B التي تستورد من الوحدة C التي تستورد من الوحدة A — مما يخلق دورة. هذه الدورات من بين أكثر المشكلات المعمارية خبثاً في قواعد الشيفرة الحديثة لأنها تآكل النمطية بصمت وتجعل الاختبار صعباً.</p>

<h2 id="strong-vs-runtime-cycles">الدورات القوية مقابل دورات وقت التشغيل</h2>
<p>يميّز AigisCode بين نوعين من التبعيات الدائرية:</p>
<ul>
<li><strong>الدورات المعمارية القوية</strong> — دورات في رسم الاستيراد الثابت حيث تكون الوحدات مقترنة هيكلياً. إعادة بناء وحدة يفرض تغييرات في الوحدات الأخرى. تُبلّغ تحت <code>graph_analysis.strong_circular_dependencies</code>.</li>
<li><strong>دورات ترتيب التحميل في وقت التشغيل</strong> — تحدث عندما يكون للوحدات استيرادات دائرية تُحل في وقت التشغيل عبر التحميل الكسول أو الاستيرادات الديناميكية. أولوية أقل وتُتتبع تحت <code>graph_analysis.circular_dependencies</code>.</li>
</ul>
<p>هذا التمييز يزيل الإيجابيات الكاذبة. يستخدم AigisCode تحليل المكونات المترابطة بقوة (SCC) عبر خوارزمية Tarjan للعثور على الدورات التي تضر فعلاً بشيفرتك.</p>

<h2 id="how-detection-works">كيف يعمل الاكتشاف</h2>
<p>تمر عملية الاكتشاف بثلاث مراحل:</p>
<ol>
<li><strong>الفهرسة</strong> — يحلل AigisCode كل ملف بمحلل tree-sitter (PHP وTypeScript وJavaScript وVue) أو tree-sitter-python ويستخرج جميع تعليمات الاستيراد.</li>
<li><strong>الرسم البياني</strong> — يُبنى رسم بياني للتبعيات على مستوى الملف بـ petgraph. تُحل أسماء الاستيراد المستعارة (مثل <code>@/</code> إلى <code>src/</code>) عبر <code>policy.json</code>.</li>
<li><strong>الاكتشاف</strong> — يشغّل محلل الرسم البياني خوارزمية Tarjan للعثور على المكونات المترابطة بقوة.</li>
</ol>

<h2 id="breaking-cycles">استراتيجيات كسر الدورات</h2>
<ul>
<li><strong>استخراج الأنواع المشتركة</strong> — انقل الواجهات إلى مجلد <code>types/</code> أو <code>contracts/</code> مخصص.</li>
<li><strong>عكس التبعيات</strong> — حدد واجهة ينفّذها الوحدة التابعة بدلاً من الاستيراد مباشرة.</li>
<li><strong>الاستيرادات الكسولة</strong> — في Python انقل الاستيرادات داخل الدوال. في TypeScript استخدم <code>import()</code> الديناميكي.</li>
<li><strong>نمط الوسيط</strong> — أدخل وحدة ثالثة تعتمد عليها كلتا الوحدتين مما يزيل الدورة المباشرة.</li>
</ul>`,
      pl: `<h2 id="what-are-circular-dependencies">Czym sa cykliczne zaleznosci?</h2>
<p>Cykliczne zaleznosci wystepuja, gdy dwa lub wiecej modulow zalezy od siebie nawzajem. AigisCode uzywa algorytmu Tarjana do wykrywania silnie spojnych komponentow, rozrozniaiac silne cykle architektoniczne od cykli runtime.</p>
<h2 id="how-detection-works">Jak dziala detekcja</h2>
<p>Detekcja przebiega w trzech etapach: indeksowanie, graf i detekcja. Raport zawiera pelna sciezke cyklu.</p>
<h2 id="breaking-cycles">Strategie przerywania cykli</h2>
<ul>
<li><strong>Wyodrebnij wspolne typy</strong> — Przenies interfejsy do dedykowanego katalogu <code>types/</code>.</li>
<li><strong>Odwrocenie zaleznosci</strong> — Zdefiniuj interfejs zamiast bezposredniego importu.</li>
<li><strong>Lazy importy</strong> — W Pythonie przenies importy do funkcji. W TypeScript uzyj dynamicznego <code>import()</code>.</li>
<li><strong>Wzorzec mediator</strong> — Wprowadz trzeci modul eliminujacy bezposredni cykl.</li>
</ul>`,
    },
    capabilities: [
      'Tarjan SCC analysis for strong cycles',
      'Strong vs. runtime cycle distinction',
      'Direct and transitive cycle detection',
      'Full cycle chain reporting',
      'petgraph graph engine',
      'Policy-aware import alias resolution',
      'Multi-language support (PHP, Python, TS, JS, Vue)',
      'Confidence scoring per finding',
    ],
    codeExample: `# Analyze your project for circular dependencies
aigiscode analyze /path/to/project

# Check strong architectural cycles
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.strong_circular_dependencies'

# Check all cycles including runtime
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.circular_dependencies'`,
    relatedSlugs: ['dead-code', 'bottlenecks', 'layer-violations'],
  },
  /* ---- 2. Dead Code Detection ---- */
  {
    slug: 'dead-code',
    icon: 'Bug',
    category: 'Code Quality',
    title: {
      en: 'Dead Code Detection',
      cs: 'Detekce mrtvého kódu',
      fr: 'Détection du code mort',
      es: 'Detección de código muerto',
      zh: '死代码检测',
      hi: 'डेड कोड पहचान',
      pt: 'Detecção de código morto',
      ar: 'اكتشاف الشيفرة الميتة',
      pl: 'Wykrywanie martwego kodu',
      bn: 'ডেড কোড ডিটেকশন',
    },
    shortDescription: {
      en: 'Eliminate unused imports, abandoned classes, unreferenced methods, and orphaned properties. AigisCode traces symbol references across your entire project to find code that nothing calls.',
      cs: 'Eliminujte nepoužívané importy, opuštěné třídy, neodkazované metody a osiřelé vlastnosti. AigisCode sleduje odkazy na symboly v celém projektu a najde kód, který nikdo nevolá.',
      fr: 'Éliminez les imports inutilisés, les classes abandonnées, les méthodes non référencées et les propriétés orphelines. AigisCode trace les références de symboles dans l\'ensemble de votre projet pour trouver le code que rien n\'appelle.',
      es: 'Elimina imports no utilizados, clases abandonadas, métodos sin referencia y propiedades huérfanas. AigisCode rastrea las referencias de símbolos en todo el proyecto para encontrar código que nada invoca.',
      zh: '消除未使用的导入、废弃的类、未引用的方法和孤立的属性。AigisCode 在整个项目中追踪符号引用，找到没有任何调用的代码。',
      hi: 'अप्रयुक्त इम्पोर्ट, परित्यक्त क्लास, अनरेफ़रेंस्ड मेथड और अनाथ प्रॉपर्टी को समाप्त करें। AigisCode आपके पूरे प्रोजेक्ट में सिंबल रेफ़रेंस को ट्रेस करके ऐसा कोड ढूंढता है जिसे कोई कॉल नहीं करता।',
      pt: 'Elimine imports não utilizados, classes abandonadas, métodos sem referência e propriedades órfãs. AigisCode rastreia referências de símbolos em todo o projeto para encontrar código que nada chama.',
      ar: 'أزل الاستيرادات غير المستخدمة والفئات المهجورة والطرق غير المرجعية والخصائص اليتيمة. يتتبع AigisCode مراجع الرموز عبر مشروعك بالكامل للعثور على الشيفرة التي لا يستدعيها شيء.',
      pl: 'Eliminuj nieuzywane importy, porzucone klasy, nieodwolywane metody i osierocone wlasciwosci. AigisCode sledzi odwolania do symboli w calym projekcie, aby znalezc kod, ktory nic nie wywoluje — z poziomami pewnosci redukujacymi falszywe alarmy.',
      bn: 'অব্যবহৃত ইমপোর্ট, পরিত্যক্ত ক্লাস, অরেফারেন্সড মেথড এবং অনাথ প্রোপার্টি নির্মূল করুন। AigisCode আপনার সম্পূর্ণ প্রজেক্ট জুড়ে সিম্বল রেফারেন্স ট্রেস করে এমন কোড খুঁজে যা কিছুই কল করে না।',
    },
    metaDescription: {
      en: 'Find and remove dead code in PHP, Python, TypeScript, and JavaScript projects. AigisCode detects unused imports, abandoned classes, orphan files, and unreferenced methods with framework-aware filtering.',
      cs: 'Najděte a odstraňte mrtvý kód v projektech PHP, Python, TypeScript a JavaScript. AigisCode detekuje nepoužívané importy, opuštěné třídy, osiřelé soubory a neodkazované metody s filtrováním podle frameworku.',
      fr: 'Trouvez et supprimez le code mort dans les projets PHP, Python, TypeScript et JavaScript. AigisCode détecte les imports inutilisés, les classes abandonnées, les fichiers orphelins et les méthodes non référencées avec un filtrage adapté aux frameworks.',
      es: 'Encuentra y elimina código muerto en proyectos PHP, Python, TypeScript y JavaScript. AigisCode detecta imports no utilizados, clases abandonadas, archivos huérfanos y métodos sin referencia con filtrado adaptado a frameworks.',
      zh: '在 PHP、Python、TypeScript 和 JavaScript 项目中查找和删除死代码。AigisCode 通过框架感知过滤检测未使用的导入、废弃的类、孤立文件和未引用的方法。',
      hi: 'PHP, Python, TypeScript और JavaScript प्रोजेक्ट में डेड कोड खोजें और हटाएं। AigisCode फ्रेमवर्क-अवेयर फ़िल्टरिंग के साथ अप्रयुक्त इम्पोर्ट, परित्यक्त क्लास, ऑर्फ़न फ़ाइलें और अनरेफ़रेंस्ड मेथड का पता लगाता है।',
      pt: 'Encontre e remova código morto em projetos PHP, Python, TypeScript e JavaScript. AigisCode detecta imports não utilizados, classes abandonadas, arquivos órfãos e métodos sem referência com filtragem adaptada a frameworks.',
      ar: 'اعثر على الشيفرة الميتة وأزلها في مشاريع PHP وPython وTypeScript وJavaScript. يكتشف AigisCode الاستيرادات غير المستخدمة والفئات المهجورة والملفات اليتيمة والطرق غير المرجعية مع تصفية واعية بالإطار.',
      pl: 'Znajdz i usun martwy kod w projektach PHP, Python, TypeScript i JavaScript. AigisCode wykrywa nieuzywane importy, porzucone klasy, osierocone pliki i nieodwolywane metody.',
      bn: 'PHP, Python, TypeScript এবং JavaScript প্রজেক্টে ডেড কোড খুঁজুন এবং সরান। AigisCode ফ্রেমওয়ার্ক-সচেতন ফিল্টারিং সহ অব্যবহৃত ইমপোর্ট, পরিত্যক্ত ক্লাস, অনাথ ফাইল এবং অরেফারেন্সড মেথড শনাক্ত করে।',
    },
    content: {
      en: `<h2 id="hidden-cost">The Hidden Cost of Dead Code</h2>
<p>Dead code is the silent tax every growing codebase pays. Unused imports inflate bundle sizes. Abandoned classes confuse new developers who waste hours understanding code that nothing calls. Orphaned files accumulate in directories, making searches slower and code reviews harder. Over time, dead code becomes a breeding ground for security vulnerabilities — code that nobody maintains is code that nobody patches.</p>
<p>The challenge is that dead code is nearly invisible at the file level. An individual import looks fine. A class might appear to be used because it follows a naming convention. Only when you analyze the <strong>entire dependency graph</strong> does the dead code reveal itself: no inbound edges means no references, which means dead code.</p>

<h2 id="what-aigiscode-finds">What AigisCode Finds</h2>
<p>AigisCode's dead code detector identifies four categories of unused code:</p>
<ul>
<li><strong>Unused imports</strong> — Import statements that bring in symbols never referenced in the file. This is the most common type of dead code and the easiest to fix. Both full imports (<code>import os</code>) and partial from-imports (<code>from typing import List, Dict</code> where only <code>List</code> is used) are detected.</li>
<li><strong>Abandoned classes</strong> — Classes that exist in the codebase but are never instantiated, extended, or referenced by any other file. These often result from incomplete feature removals or deprecated functionality.</li>
<li><strong>Unreferenced methods</strong> — Private methods on classes that are never called. AigisCode checks the scope and visibility to reduce false positives on methods that may be accessed through reflection.</li>
<li><strong>Orphaned properties</strong> — Private class properties that are declared but never read or written outside their declaration.</li>
</ul>

<h2 id="framework-awareness">Framework-Aware Filtering</h2>
<p>Not all unreferenced code is truly dead. Modern frameworks use conventions, decorators, and runtime registration that create invisible references:</p>
<ul>
<li><strong>PHP attributes</strong> — Methods annotated with <code>#[Override]</code>, <code>#[Route]</code>, or similar attributes are recognized as framework-referenced.</li>
<li><strong>Dependency injection type hints</strong> — Classes referenced only as DI container type hints are not flagged as abandoned.</li>
<li><strong>Test classes</strong> — Test files and test methods are excluded from dead code detection since they are invoked by the test runner, not by application code.</li>
<li><strong>Plugin profiles</strong> — The Django, Laravel, and WordPress plugins mark framework-conventional files (migrations, service providers, hook callbacks) as entry points.</li>
</ul>
<p>Framework-aware filtering is driven by the policy system, not hardcoded into the detector. You can customize entry point patterns via <code>policy.json</code>:</p>
<pre><code>{
  "dead_code": {
    "abandoned_entry_patterns": [
      "src/commands/**/*.ts",
      "database/migrations/**/*.php",
      "tests/fixtures/**/*"
    ]
  }
}</code></pre>

<h2 id="detection-pipeline">Detection Pipeline</h2>
<p>Dead code detection runs during the Detect stage of AigisCode's pipeline:</p>
<ol>
<li><strong>Symbol extraction</strong> — The indexer parses every file to extract all exported symbols: classes, functions, constants, type definitions, and re-exports.</li>
<li><strong>Reference tracing</strong> — The graph builder tracks every import statement to determine which symbols are actually consumed by other files.</li>
<li><strong>Orphan analysis</strong> — Files with zero inbound edges in the dependency graph are flagged as orphans. Entry point patterns from your policy are excluded.</li>
<li><strong>Confidence scoring</strong> — Each finding gets a confidence level. Unused imports are high confidence. Abandoned classes in a DI-heavy framework are medium confidence.</li>
</ol>

<h2 id="multi-language-support">Multi-Language Support</h2>
<p>AigisCode detects dead code across all supported languages in a single analysis pass:</p>
<ul>
<li><strong>PHP</strong> — tree-sitter parsing, handles <code>use</code> statements, class method visibility, trait usage</li>
<li><strong>Python</strong> — AST parsing, handles <code>import</code>/<code>from...import</code>, <code>TYPE_CHECKING</code> blocks, star imports</li>
<li><strong>TypeScript/JavaScript</strong> — tree-sitter parsing, handles ESM/CJS imports, re-exports, barrel files</li>
<li><strong>Vue</strong> — tree-sitter parsing of <code>&lt;script&gt;</code> blocks with proper SFC handling</li>
</ul>`,
      cs: `<h2 id="hidden-cost">Skryté náklady mrtvého kódu</h2>
<p>Mrtvý kód je tichá daň, kterou platí každá rostoucí kódová základna. Nepoužívané importy zvětšují velikost bundlů. Opuštěné třídy matou nové vývojáře. Osiřelé soubory se hromadí v adresářích a zpomalují vyhledávání. Mrtvý kód se časem stává živnou půdou pro bezpečnostní zranitelnosti — kód, který nikdo neudržuje, nikdo neopravuje.</p>

<h2 id="what-aigiscode-finds">Co AigisCode nachází</h2>
<p>Detektor mrtvého kódu identifikuje čtyři kategorie:</p>
<ul>
<li><strong>Nepoužívané importy</strong> — Importní příkazy přinášející symboly, které nejsou v souboru nikdy odkazovány.</li>
<li><strong>Opuštěné třídy</strong> — Třídy, které nikdy nejsou instanciovány, rozšířeny ani odkazovány jiným souborem.</li>
<li><strong>Neodkazované metody</strong> — Privátní metody tříd, které nejsou nikdy volány.</li>
<li><strong>Osiřelé vlastnosti</strong> — Privátní vlastnosti tříd, které jsou deklarovány, ale nikdy čteny ani zapisovány.</li>
</ul>

<h2 id="framework-awareness">Filtrování s ohledem na framework</h2>
<p>Ne všechen neodkazovaný kód je skutečně mrtvý. Moderní frameworky používají konvence a dekorátory, které vytvářejí neviditelné reference. AigisCode rozpoznává PHP atributy, DI type hinty a testovací třídy. Pluginy pro Django, Laravel a WordPress označují konvenční soubory frameworku jako vstupní body.</p>

<h2 id="detection-pipeline">Detekční pipeline</h2>
<ol>
<li><strong>Extrakce symbolů</strong> — Indexer parsuje každý soubor a extrahuje všechny exportované symboly.</li>
<li><strong>Trasování referencí</strong> — Builder grafu sleduje každý importní příkaz.</li>
<li><strong>Analýza orphanů</strong> — Soubory s nulovým počtem příchozích hran jsou označeny jako orphany.</li>
<li><strong>Bodování důvěryhodnosti</strong> — Každý nález dostává úroveň důvěryhodnosti.</li>
</ol>

<h2 id="multi-language-support">Podpora více jazyků</h2>
<p>AigisCode detekuje mrtvý kód napříč všemi podporovanými jazyky v jednom průchodu analýzy: PHP (tree-sitter), Python (AST), TypeScript/JavaScript (tree-sitter) a Vue (tree-sitter).</p>`,
      fr: `<h2 id="hidden-cost">Le cout cache du code mort</h2>
<p>Le code mort est la taxe silencieuse que paie chaque base de code en croissance. Les imports inutilises gonflent la taille des bundles. Les classes abandonnees deroutent les nouveaux developpeurs. Les fichiers orphelins s'accumulent dans les repertoires. Au fil du temps, le code mort devient un terrain fertile pour les vulnerabilites de securite.</p>

<h2 id="what-aigiscode-finds">Ce que trouve AigisCode</h2>
<p>Le detecteur de code mort identifie quatre categories :</p>
<ul>
<li><strong>Imports inutilises</strong> — Instructions d'import amenant des symboles jamais references dans le fichier.</li>
<li><strong>Classes abandonnees</strong> — Classes jamais instanciees, etendues ou referencees par un autre fichier.</li>
<li><strong>Methodes non referencees</strong> — Methodes privees de classes jamais appelees.</li>
<li><strong>Proprietes orphelines</strong> — Proprietes privees de classes declarees mais jamais lues ni ecrites.</li>
</ul>

<h2 id="framework-awareness">Filtrage adapte aux frameworks</h2>
<p>Tout le code non reference n'est pas vraiment mort. Les frameworks modernes utilisent des conventions et decorateurs qui creent des references invisibles. AigisCode reconnait les attributs PHP, les type hints DI et les classes de test. Les plugins Django, Laravel et WordPress marquent les fichiers conventionnels comme points d'entree.</p>

<h2 id="detection-pipeline">Pipeline de detection</h2>
<ol>
<li><strong>Extraction de symboles</strong> — L'indexeur analyse chaque fichier et extrait tous les symboles exportes.</li>
<li><strong>Tracage des references</strong> — Le constructeur de graphe suit chaque instruction d'import.</li>
<li><strong>Analyse des orphelins</strong> — Les fichiers sans aretes entrantes sont signales comme orphelins.</li>
<li><strong>Score de confiance</strong> — Chaque resultat recoit un niveau de confiance.</li>
</ol>

<h2 id="multi-language-support">Support multi-langage</h2>
<p>AigisCode detecte le code mort dans tous les langages supportes en une seule passe : PHP (tree-sitter), Python (AST), TypeScript/JavaScript (tree-sitter) et Vue (tree-sitter).</p>`,
      es: `<h2 id="hidden-cost">El costo oculto del codigo muerto</h2>
<p>El codigo muerto es el impuesto silencioso que paga cada base de codigo en crecimiento. Los imports no utilizados inflan el tamano de los bundles. Las clases abandonadas confunden a los nuevos desarrolladores. Los archivos huerfanos se acumulan en los directorios. Con el tiempo, el codigo muerto se convierte en un caldo de cultivo para vulnerabilidades de seguridad.</p>

<h2 id="what-aigiscode-finds">Que encuentra AigisCode</h2>
<p>El detector de codigo muerto identifica cuatro categorias:</p>
<ul>
<li><strong>Imports no utilizados</strong> — Declaraciones de importacion que traen simbolos nunca referenciados en el archivo.</li>
<li><strong>Clases abandonadas</strong> — Clases que nunca son instanciadas, extendidas o referenciadas por otro archivo.</li>
<li><strong>Metodos no referenciados</strong> — Metodos privados de clases que nunca son llamados.</li>
<li><strong>Propiedades huerfanas</strong> — Propiedades privadas de clases declaradas pero nunca leidas ni escritas.</li>
</ul>

<h2 id="framework-awareness">Filtrado consciente del framework</h2>
<p>No todo el codigo sin referencia esta realmente muerto. Los frameworks modernos usan convenciones y decoradores que crean referencias invisibles. AigisCode reconoce atributos PHP, type hints de DI y clases de prueba. Los plugins de Django, Laravel y WordPress marcan los archivos convencionales como puntos de entrada.</p>

<h2 id="detection-pipeline">Pipeline de deteccion</h2>
<ol>
<li><strong>Extraccion de simbolos</strong> — El indexador analiza cada archivo y extrae todos los simbolos exportados.</li>
<li><strong>Rastreo de referencias</strong> — El constructor de grafos sigue cada declaracion de importacion.</li>
<li><strong>Analisis de huerfanos</strong> — Los archivos sin aristas entrantes se marcan como huerfanos.</li>
<li><strong>Puntuacion de confianza</strong> — Cada hallazgo recibe un nivel de confianza.</li>
</ol>

<h2 id="multi-language-support">Soporte multi-lenguaje</h2>
<p>AigisCode detecta codigo muerto en todos los lenguajes soportados en una sola pasada: PHP (tree-sitter), Python (AST), TypeScript/JavaScript (tree-sitter) y Vue (tree-sitter).</p>`,
      zh: `<h2 id="hidden-cost">死代码的隐性成本</h2>
<p>死代码是每个增长中的代码库都要支付的无形税收。未使用的导入膨胀了包体积。废弃的类使新开发者困惑。孤立文件在目录中积累，使搜索变慢。随着时间推移，死代码成为安全漏洞的温床——没人维护的代码就是没人修补的代码。</p>

<h2 id="what-aigiscode-finds">AigisCode发现什么</h2>
<p>死代码检测器识别四类未使用代码：</p>
<ul>
<li><strong>未使用的导入</strong> — 引入但从未在文件中引用的符号的导入语句。</li>
<li><strong>废弃的类</strong> — 从未被实例化、继承或被其他文件引用的类。</li>
<li><strong>未引用的方法</strong> — 从未被调用的类私有方法。</li>
<li><strong>孤立的属性</strong> — 声明了但从未被读取或写入的类私有属性。</li>
</ul>

<h2 id="framework-awareness">框架感知过滤</h2>
<p>并非所有未引用的代码都是真正的死代码。现代框架使用约定和装饰器创建不可见的引用。AigisCode识别PHP属性、DI类型提示和测试类。Django、Laravel和WordPress插件将框架约定文件标记为入口点。</p>

<h2 id="detection-pipeline">检测流程</h2>
<ol>
<li><strong>符号提取</strong> — 索引器解析每个文件，提取所有导出的符号。</li>
<li><strong>引用追踪</strong> — 图构建器跟踪每个导入语句。</li>
<li><strong>孤立分析</strong> — 零入度的文件被标记为孤立文件。</li>
<li><strong>置信度评分</strong> — 每个发现获得一个置信度级别。</li>
</ol>

<h2 id="multi-language-support">多语言支持</h2>
<p>AigisCode在一次分析中检测所有支持语言的死代码：PHP（tree-sitter）、Python（AST）、TypeScript/JavaScript（tree-sitter）和Vue（tree-sitter）。</p>`,
      hi: `<h2 id="hidden-cost">डेड कोड की छिपी लागत</h2>
<p>डेड कोड वह मूक कर है जो हर बढ़ती कोडबेस को चुकाना पड़ता है। अप्रयुक्त इम्पोर्ट बंडल साइज़ बढ़ाते हैं। परित्यक्त क्लास नए डेवलपर्स को भ्रमित करती हैं। अनाथ फ़ाइलें डायरेक्टरी में जमा होती हैं। समय के साथ, डेड कोड सुरक्षा कमजोरियों का अड्डा बन जाता है।</p>

<h2 id="what-aigiscode-finds">AigisCode क्या खोजता है</h2>
<p>डेड कोड डिटेक्टर चार श्रेणियां पहचानता है:</p>
<ul>
<li><strong>अप्रयुक्त इम्पोर्ट</strong> — ऐसे इम्पोर्ट स्टेटमेंट जो फ़ाइल में कभी रेफ़रेंस नहीं किए गए सिंबल लाते हैं।</li>
<li><strong>परित्यक्त क्लास</strong> — ऐसी क्लास जो कभी इंस्टैंशिएट, एक्सटेंड या किसी अन्य फ़ाइल द्वारा रेफ़रेंस नहीं की गईं।</li>
<li><strong>अनरेफ़रेंस्ड मेथड</strong> — क्लास के प्राइवेट मेथड जो कभी कॉल नहीं किए गए।</li>
<li><strong>अनाथ प्रॉपर्टी</strong> — क्लास की प्राइवेट प्रॉपर्टी जो डिक्लेयर की गई लेकिन कभी पढ़ी या लिखी नहीं गई।</li>
</ul>

<h2 id="framework-awareness">फ्रेमवर्क-अवेयर फ़िल्टरिंग</h2>
<p>सभी अनरेफ़रेंस्ड कोड वास्तव में डेड नहीं होता। आधुनिक फ्रेमवर्क कन्वेंशन और डेकोरेटर्स का उपयोग करते हैं जो अदृश्य रेफ़रेंस बनाते हैं। AigisCode PHP एट्रिब्यूट्स, DI टाइप हिंट्स और टेस्ट क्लास को पहचानता है। Django, Laravel और WordPress प्लगइन्स फ्रेमवर्क कन्वेंशनल फ़ाइलों को एंट्री पॉइंट के रूप में चिह्नित करते हैं।</p>

<h2 id="detection-pipeline">डिटेक्शन पाइपलाइन</h2>
<ol>
<li><strong>सिंबल एक्सट्रैक्शन</strong> — इंडेक्सर हर फ़ाइल को पार्स करके सभी एक्सपोर्टेड सिंबल एक्सट्रैक्ट करता है।</li>
<li><strong>रेफ़रेंस ट्रेसिंग</strong> — ग्राफ बिल्डर हर इम्पोर्ट स्टेटमेंट को ट्रैक करता है।</li>
<li><strong>ऑर्फ़न एनालिसिस</strong> — शून्य इनबाउंड एज वाली फ़ाइलें ऑर्फ़न के रूप में फ़्लैग की जाती हैं।</li>
<li><strong>कॉन्फिडेंस स्कोरिंग</strong> — हर फ़ाइंडिंग को एक कॉन्फिडेंस लेवल मिलता है।</li>
</ol>

<h2 id="multi-language-support">मल्टी-लैंग्वेज सपोर्ट</h2>
<p>AigisCode एक ही एनालिसिस पास में सभी समर्थित भाषाओं में डेड कोड का पता लगाता है: PHP (tree-sitter), Python (AST), TypeScript/JavaScript (tree-sitter) और Vue (tree-sitter)।</p>`,
      pt: `<h2 id="hidden-cost">O custo oculto do codigo morto</h2>
<p>Codigo morto e o imposto silencioso que toda base de codigo em crescimento paga. Imports nao utilizados inflam o tamanho dos bundles. Classes abandonadas confundem novos desenvolvedores. Arquivos orfaos se acumulam nos diretorios. Com o tempo, codigo morto se torna terreno fertil para vulnerabilidades de seguranca.</p>

<h2 id="what-aigiscode-finds">O que o AigisCode encontra</h2>
<p>O detector de codigo morto identifica quatro categorias:</p>
<ul>
<li><strong>Imports nao utilizados</strong> — Declaracoes de importacao que trazem simbolos nunca referenciados no arquivo.</li>
<li><strong>Classes abandonadas</strong> — Classes que nunca sao instanciadas, estendidas ou referenciadas por outro arquivo.</li>
<li><strong>Metodos nao referenciados</strong> — Metodos privados de classes que nunca sao chamados.</li>
<li><strong>Propriedades orfas</strong> — Propriedades privadas de classes declaradas mas nunca lidas ou escritas.</li>
</ul>

<h2 id="framework-awareness">Filtragem adaptada a frameworks</h2>
<p>Nem todo codigo sem referencia esta realmente morto. Frameworks modernos usam convencoes e decoradores que criam referencias invisiveis. AigisCode reconhece atributos PHP, type hints de DI e classes de teste. Os plugins Django, Laravel e WordPress marcam arquivos convencionais como pontos de entrada.</p>

<h2 id="detection-pipeline">Pipeline de deteccao</h2>
<ol>
<li><strong>Extracao de simbolos</strong> — O indexador analisa cada arquivo e extrai todos os simbolos exportados.</li>
<li><strong>Rastreamento de referencias</strong> — O construtor de grafo segue cada declaracao de importacao.</li>
<li><strong>Analise de orfaos</strong> — Arquivos sem arestas de entrada sao marcados como orfaos.</li>
<li><strong>Pontuacao de confianca</strong> — Cada descoberta recebe um nivel de confianca.</li>
</ol>

<h2 id="multi-language-support">Suporte multi-linguagem</h2>
<p>AigisCode detecta codigo morto em todas as linguagens suportadas em uma unica passada: PHP (tree-sitter), Python (AST), TypeScript/JavaScript (tree-sitter) e Vue (tree-sitter).</p>`,
      ar: `<h2 id="hidden-cost">التكلفة الخفية للشيفرة الميتة</h2>
<p>الشيفرة الميتة هي الضريبة الصامتة التي تدفعها كل قاعدة شيفرة متنامية. تضخّم الاستيرادات غير المستخدمة أحجام الحزم. تربك الفئات المهجورة المطورين الجدد. تتراكم الملفات اليتيمة في المجلدات. مع مرور الوقت تصبح الشيفرة الميتة أرضاً خصبة للثغرات الأمنية.</p>

<h2 id="what-aigiscode-finds">ما يكتشفه AigisCode</h2>
<p>يحدد كاشف الشيفرة الميتة أربع فئات:</p>
<ul>
<li><strong>الاستيرادات غير المستخدمة</strong> — تعليمات استيراد تجلب رموزاً لا يُشار إليها أبداً في الملف.</li>
<li><strong>الفئات المهجورة</strong> — فئات لا تُنشأ أو تُوسّع أو يُشار إليها من ملف آخر أبداً.</li>
<li><strong>الطرق غير المرجعية</strong> — طرق خاصة بالفئات لا تُستدعى أبداً.</li>
<li><strong>الخصائص اليتيمة</strong> — خصائص خاصة بالفئات مُعلنة لكن لا تُقرأ أو تُكتب أبداً.</li>
</ul>

<h2 id="framework-awareness">تصفية واعية بالإطار</h2>
<p>ليست كل الشيفرة غير المرجعية ميتة حقاً. تستخدم الأطر الحديثة اتفاقيات ومزخرفات تخلق مراجع غير مرئية. يتعرف AigisCode على سمات PHP وتلميحات أنواع حقن التبعيات وفئات الاختبار. تحدد إضافات Django وLaravel وWordPress الملفات الاتفاقية كنقاط دخول.</p>

<h2 id="detection-pipeline">خط أنابيب الاكتشاف</h2>
<ol>
<li><strong>استخراج الرموز</strong> — يحلل المفهرس كل ملف ويستخرج جميع الرموز المصدّرة.</li>
<li><strong>تتبع المراجع</strong> — يتبع باني الرسم البياني كل تعليمة استيراد.</li>
<li><strong>تحليل الملفات اليتيمة</strong> — تُحدد الملفات بدون حواف واردة كملفات يتيمة.</li>
<li><strong>تسجيل الثقة</strong> — تحصل كل نتيجة على مستوى ثقة.</li>
</ol>

<h2 id="multi-language-support">دعم لغات متعددة</h2>
<p>يكتشف AigisCode الشيفرة الميتة عبر جميع اللغات المدعومة في تمريرة واحدة: PHP (tree-sitter) وPython (AST) وTypeScript/JavaScript (tree-sitter) وVue (tree-sitter).</p>`,
      pl: `<h2 id="the-cost-of-dead-code">Ukryty koszt martwego kodu</h2>
<p>Martwy kod gromadzi sie po cichu. AigisCode sledzi odwolania do symboli w calym projekcie, identyfikujac nieuzywane importy, porzucone klasy, nieodwolywane metody i osierocone pliki z poziomami pewnosci.</p>
<h2 id="policy-customization">Dostosowanie polityk</h2>
<p>Punkty wejscia mozna zdefiniowac w politykach, aby nie byly oznaczane jako sieroty.</p>`,
    },
    capabilities: [
      'Unused import detection',
      'Abandoned class detection',
      'Unreferenced private method detection',
      'Orphaned property detection',
      'Framework attribute recognition (#[Override], decorators)',
      'DI type hint awareness',
      'Test class exclusion',
      'Policy-driven entry point patterns',
    ],
    codeExample: `# Analyze your project for dead code
aigiscode analyze /path/to/project

# View all dead code findings
cat .aigiscode/deterministic-analysis.json | jq '.dead_code'

# Filter by type
cat .aigiscode/deterministic-analysis.json | jq '.dead_code[] | select(.type == "unused_import")'

# Customize entry patterns in policy
cat > .aigiscode/policy.json << EOF
{
  "dead_code": {
    "abandoned_entry_patterns": ["src/commands/**/*.ts"]
  }
}
EOF`,
    relatedSlugs: ['orphan-detection', 'circular-dependencies', 'ai-review'],
  },
  /* ---- 3. Hardwiring Detection ---- */
  {
    slug: 'hardwiring',
    icon: 'Lightning',
    category: 'Code Quality',
    title: {
      en: 'Hardwiring Detection',
      cs: 'Detekce hardcoded hodnot',
      fr: 'Détection des valeurs codées en dur',
      es: 'Detección de valores codificados',
      zh: '硬编码值检测',
      hi: 'हार्डवायरिंग पहचान',
      pt: 'Detecção de valores hardcoded',
      ar: 'اكتشاف القيم الثابتة',
      pl: 'Wykrywanie wartosci zakodowanych na sztywno',
      bn: 'হার্ডওয়্যারিং ডিটেকশন',
    },
    shortDescription: {
      en: 'Catch magic strings, repeated literals, hardcoded IPs and URLs, and environment variable access outside config files. AigisCode finds configuration values scattered across your codebase that should be centralized.',
      cs: 'Zachyťte magické řetězce, opakované literály, natvrdo zapsané IP adresy a URL a přístup k proměnným prostředí mimo konfigurační soubory. AigisCode najde konfigurační hodnoty rozptýlené po kódové základně, které by měly být centralizovány.',
      fr: 'Détectez les chaînes magiques, les littéraux répétés, les adresses IP et URL codées en dur, et l\'accès aux variables d\'environnement en dehors des fichiers de configuration. AigisCode trouve les valeurs de configuration dispersées dans votre base de code qui devraient être centralisées.',
      es: 'Detecta cadenas mágicas, literales repetidos, IPs y URLs codificadas y acceso a variables de entorno fuera de archivos de configuración. AigisCode encuentra valores de configuración dispersos en tu código que deberían estar centralizados.',
      zh: '捕获魔法字符串、重复字面量、硬编码的 IP 和 URL 以及配置文件之外的环境变量访问。AigisCode 找到分散在代码库中应该集中管理的配置值。',
      hi: 'मैजिक स्ट्रिंग, दोहराए गए लिटरल, हार्डकोडेड IP और URL, और कॉन्फ़िग फ़ाइलों के बाहर एनवायरनमेंट वेरिएबल एक्सेस को पकड़ें। AigisCode आपके कोडबेस में बिखरे कॉन्फ़िगरेशन मानों को ढूंढता है जिन्हें केंद्रीकृत किया जाना चाहिए।',
      pt: 'Detecte strings mágicas, literais repetidos, IPs e URLs hardcoded e acesso a variáveis de ambiente fora de arquivos de configuração. AigisCode encontra valores de configuração espalhados pela base de código que deveriam ser centralizados.',
      ar: 'اكتشف السلاسل السحرية والقيم الحرفية المتكررة وعناوين IP وURLs الثابتة والوصول لمتغيرات البيئة خارج ملفات التكوين. يجد AigisCode قيم التكوين المبعثرة في قاعدة شيفرتك التي يجب تجميعها.',
      pl: 'Wychwytuj magiczne stringi, powtarzajace sie literaly, zakodowane na sztywno adresy IP i URL-e oraz dostep do zmiennych srodowiskowych poza plikami konfiguracyjnymi. Detektor zakodowanych wartosci AigisCode respektuje wylaczenia polityk, redukujac falszywe alarmy we wzorcach konwencjonalnych dla frameworkow.',
      bn: 'ম্যাজিক স্ট্রিং, পুনরাবৃত্ত লিটারেল, হার্ডকোডেড IP ও URL এবং কনফিগ ফাইলের বাইরে এনভায়রনমেন্ট ভেরিয়েবল অ্যাক্সেস ধরুন। AigisCode আপনার কোডবেস জুড়ে ছড়িয়ে থাকা কনফিগারেশন ভ্যালু খুঁজে যা কেন্দ্রীভূত হওয়া উচিত।',
    },
    metaDescription: {
      en: 'Detect hardwired values in PHP, Python, TypeScript, and JavaScript codebases. AigisCode finds magic strings, repeated literals, hardcoded IPs/URLs, and env access outside config files.',
      cs: 'Detekujte natvrdo zapsané hodnoty v kódových základnách PHP, Python, TypeScript a JavaScript. AigisCode najde magické řetězce, opakované literály, natvrdo zapsané IP/URL a přístup k proměnným prostředí mimo konfigurační soubory.',
      fr: 'Détectez les valeurs codées en dur dans les bases de code PHP, Python, TypeScript et JavaScript. AigisCode trouve les chaînes magiques, les littéraux répétés, les IP/URL codées en dur et l\'accès aux variables d\'environnement en dehors des fichiers de configuration.',
      es: 'Detecta valores codificados en bases de código PHP, Python, TypeScript y JavaScript. AigisCode encuentra cadenas mágicas, literales repetidos, IPs/URLs codificadas y acceso a variables de entorno fuera de archivos de configuración.',
      zh: '检测 PHP、Python、TypeScript 和 JavaScript 代码库中的硬编码值。AigisCode 发现魔法字符串、重复字面量、硬编码的 IP/URL 以及配置文件之外的环境变量访问。',
      hi: 'PHP, Python, TypeScript और JavaScript कोडबेस में हार्डवायर्ड वैल्यू का पता लगाएं। AigisCode मैजिक स्ट्रिंग, दोहराए गए लिटरल, हार्डकोडेड IP/URL और कॉन्फ़िग फ़ाइलों के बाहर env एक्सेस खोजता है।',
      pt: 'Detecte valores hardcoded em bases de código PHP, Python, TypeScript e JavaScript. AigisCode encontra strings mágicas, literais repetidos, IPs/URLs hardcoded e acesso a variáveis de ambiente fora de arquivos de configuração.',
      ar: 'اكتشف القيم الثابتة في قواعد شيفرة PHP وPython وTypeScript وJavaScript. يجد AigisCode السلاسل السحرية والقيم الحرفية المتكررة وعناوين IP/URLs الثابتة والوصول لمتغيرات البيئة خارج ملفات التكوين.',
      pl: 'Wykrywaj zakodowane wartosci w bazach kodu PHP, Python, TypeScript i JavaScript. AigisCode znajduje magiczne stringi, powtarzajace sie literaly, zakodowane adresy IP i dostep do zmiennych srodowiskowych poza plikami konfiguracyjnymi.',
      bn: 'PHP, Python, TypeScript এবং JavaScript কোডবেসে হার্ডওয়্যার্ড ভ্যালু শনাক্ত করুন। AigisCode ম্যাজিক স্ট্রিং, পুনরাবৃত্ত লিটারেল, হার্ডকোডেড IP/URL এবং কনফিগ ফাইলের বাইরে env অ্যাক্সেস খুঁজে।',
    },
    content: {
      en: `<h2 id="what-is-hardwiring">What Is Hardwiring?</h2>
<p>Hardwiring is the practice of embedding configuration values directly in application code instead of centralizing them in configuration files, environment variables, or settings modules. A database URL hardcoded in a service class, an API endpoint repeated in five different files, an IP address buried in a middleware function — these are all hardwired values that create maintenance nightmares.</p>
<p>Hardwired values cause problems because they violate the single-source-of-truth principle. When a value changes (and configuration values always change), you have to find and update every occurrence. Miss one and you have a bug. Worse, hardwired credentials are a security risk if the code is ever exposed.</p>

<h2 id="four-categories">Four Categories of Hardwiring</h2>
<p>AigisCode detects four distinct types of hardwired values:</p>
<ul>
<li><strong>Magic strings</strong> — String literals that encode business logic, feature flags, or status values. Examples: <code>"active"</code>, <code>"premium"</code>, <code>"admin"</code> used as comparisons scattered across multiple files.</li>
<li><strong>Repeated literals</strong> — The same literal value appearing 3 or more times across different directories. This threshold is configurable via <code>repeated_literal_min_occurrences</code> in your policy. When a value is repeated that often, it should be a named constant.</li>
<li><strong>Hardcoded IPs and URLs</strong> — IP addresses (<code>192.168.1.1</code>), URLs (<code>https://api.example.com/v2</code>), and connection strings embedded in application code rather than read from configuration.</li>
<li><strong>Environment access outside config</strong> — Calls to <code>os.environ.get()</code>, <code>process.env.</code>, <code>getenv()</code>, or <code>$_ENV</code> appearing in files outside your designated config directory. Environment access should be centralized in a single config module.</li>
</ul>

<h2 id="framework-awareness">Framework-Aware Filtering</h2>
<p>AigisCode applies intelligent filtering to reduce false positives:</p>
<ul>
<li><strong>Test files</strong> — String literals in test files are not flagged because test fixtures naturally contain hardcoded values.</li>
<li><strong>Config files</strong> — Files in recognized config directories (<code>config/</code>, <code>settings/</code>) are excluded from env-access warnings since that is where env access belongs.</li>
<li><strong>Framework conventions</strong> — Laravel's <code>env()</code> calls in <code>config/*.php</code> are expected. Django's <code>settings.py</code> naturally accesses environment variables. WordPress's <code>wp-config.php</code> is meant to contain configuration.</li>
<li><strong>Skip patterns</strong> — Configure <code>skip_path_patterns</code> in your policy to exclude directories from hardwiring scans entirely.</li>
</ul>
<pre><code>{
  "hardwiring": {
    "repeated_literal_min_occurrences": 4,
    "skip_path_patterns": [
      "app/Console/*",
      "tests/**/*",
      "database/seeders/*"
    ]
  }
}</code></pre>

<h2 id="detection-logic">How Detection Works</h2>
<p>The hardwiring detector scans every parsed file and applies pattern-based analysis:</p>
<ol>
<li><strong>Literal extraction</strong> — All string and numeric literals are extracted from the AST of each file.</li>
<li><strong>Cross-file aggregation</strong> — Literals are grouped across the entire project. When the same literal appears in 3+ files across different directories, it is flagged as a repeated literal.</li>
<li><strong>Pattern matching</strong> — IP addresses, URLs, and connection strings are detected via regex patterns applied to extracted literals.</li>
<li><strong>Env-access tracking</strong> — Calls to environment-reading functions are tracked and flagged when they appear outside designated config files.</li>
</ol>

<h2 id="fixing-hardwiring">Fixing Hardwired Values</h2>
<p>The recommended approach for fixing hardwired values depends on the type:</p>
<ul>
<li><strong>Repeated literals</strong> — Extract to a named constant in a shared constants module.</li>
<li><strong>Hardcoded URLs/IPs</strong> — Move to environment variables or a configuration file.</li>
<li><strong>Scattered env access</strong> — Centralize all <code>os.environ</code> / <code>process.env</code> reads in a single config module and import values from there.</li>
<li><strong>Magic strings</strong> — Replace with enums or named constants that provide type safety and IDE autocompletion.</li>
</ul>`,
      cs: `<h2 id="what-is-hardwiring">Co je hardwiring?</h2>
<p>Hardwiring je praxe vkládání konfiguračních hodnot přímo do aplikačního kódu místo jejich centralizace v konfiguračních souborech nebo proměnných prostředí. URL databáze natvrdo zapsaná ve třídě služby, API endpoint opakovaný v pěti souborech, IP adresa ukrytá ve funkci middleware — to vše jsou hardwired hodnoty, které vytvářejí noční můry údržby.</p>

<h2 id="four-categories">Čtyři kategorie hardwiringu</h2>
<ul>
<li><strong>Magické řetězce</strong> — Řetězcové literály kódující obchodní logiku nebo stavové hodnoty rozptýlené napříč soubory.</li>
<li><strong>Opakované literály</strong> — Stejná literální hodnota objevující se 3 nebo vícekrát v různých adresářích. Práh je konfigurovatelný přes <code>repeated_literal_min_occurrences</code>.</li>
<li><strong>Natvrdo zapsané IP a URL</strong> — IP adresy, URL a připojovací řetězce vložené v aplikačním kódu místo čtení z konfigurace.</li>
<li><strong>Přístup k prostředí mimo config</strong> — Volání <code>os.environ.get()</code>, <code>process.env.</code>, <code>getenv()</code> nebo <code>$_ENV</code> mimo konfigurační adresář.</li>
</ul>

<h2 id="framework-awareness">Filtrování s ohledem na framework</h2>
<p>AigisCode aplikuje inteligentní filtrování: testovací soubory jsou vynechány, konfigurační adresáře jsou vyloučeny z varování env-access a frameworkové konvence jsou respektovány. Konfigurujte <code>skip_path_patterns</code> ve vaší politice.</p>

<h2 id="fixing-hardwiring">Oprava hardwired hodnot</h2>
<ul>
<li><strong>Opakované literály</strong> — Extrahujte do pojmenované konstanty ve sdíleném modulu.</li>
<li><strong>Natvrdo zapsané URL/IP</strong> — Přesuňte do proměnných prostředí nebo konfiguračního souboru.</li>
<li><strong>Rozptýlený přístup k env</strong> — Centralizujte čtení <code>os.environ</code> / <code>process.env</code> v jednom konfiguračním modulu.</li>
<li><strong>Magické řetězce</strong> — Nahraďte výčtovými typy nebo pojmenovanými konstantami.</li>
</ul>`,
      fr: `<h2 id="what-is-hardwiring">Qu'est-ce que le hardwiring ?</h2>
<p>Le hardwiring est la pratique d'incorporer des valeurs de configuration directement dans le code applicatif au lieu de les centraliser dans des fichiers de configuration ou des variables d'environnement. Une URL de base de donnees codee en dur dans une classe de service, un endpoint API repete dans cinq fichiers, une adresse IP enfouie dans une fonction middleware — ce sont autant de valeurs codees en dur qui creent des cauchemars de maintenance.</p>

<h2 id="four-categories">Quatre categories de hardwiring</h2>
<ul>
<li><strong>Chaines magiques</strong> — Litteraux de chaines encodant la logique metier ou des valeurs d'etat dispersees dans les fichiers.</li>
<li><strong>Litteraux repetes</strong> — La meme valeur litterale apparaissant 3 fois ou plus dans differents repertoires. Le seuil est configurable via <code>repeated_literal_min_occurrences</code>.</li>
<li><strong>IPs et URLs codees en dur</strong> — Adresses IP, URLs et chaines de connexion incorporees dans le code au lieu d'etre lues depuis la configuration.</li>
<li><strong>Acces env hors config</strong> — Appels a <code>os.environ.get()</code>, <code>process.env.</code>, <code>getenv()</code> ou <code>$_ENV</code> en dehors du repertoire de configuration.</li>
</ul>

<h2 id="framework-awareness">Filtrage adapte aux frameworks</h2>
<p>AigisCode applique un filtrage intelligent : les fichiers de test sont ignores, les repertoires de configuration sont exclus des avertissements d'acces env, et les conventions de framework sont respectees. Configurez <code>skip_path_patterns</code> dans votre politique.</p>

<h2 id="fixing-hardwiring">Corriger les valeurs codees en dur</h2>
<ul>
<li><strong>Litteraux repetes</strong> — Extraire dans une constante nommee dans un module partage.</li>
<li><strong>URLs/IPs codees en dur</strong> — Deplacer vers des variables d'environnement ou un fichier de configuration.</li>
<li><strong>Acces env disperse</strong> — Centraliser toutes les lectures <code>os.environ</code> / <code>process.env</code> dans un seul module de configuration.</li>
<li><strong>Chaines magiques</strong> — Remplacer par des enums ou des constantes nommees.</li>
</ul>`,
      es: `<h2 id="what-is-hardwiring">Que es el hardwiring?</h2>
<p>El hardwiring es la practica de incrustar valores de configuracion directamente en el codigo de la aplicacion en lugar de centralizarlos en archivos de configuracion o variables de entorno. Una URL de base de datos hardcodeada en una clase de servicio, un endpoint de API repetido en cinco archivos, una direccion IP enterrada en una funcion middleware — todos son valores hardcoded que crean pesadillas de mantenimiento.</p>

<h2 id="four-categories">Cuatro categorias de hardwiring</h2>
<ul>
<li><strong>Cadenas magicas</strong> — Literales de cadena que codifican logica de negocio o valores de estado dispersos en los archivos.</li>
<li><strong>Literales repetidos</strong> — El mismo valor literal apareciendo 3 o mas veces en diferentes directorios. El umbral es configurable via <code>repeated_literal_min_occurrences</code>.</li>
<li><strong>IPs y URLs hardcodeadas</strong> — Direcciones IP, URLs y cadenas de conexion incrustadas en el codigo en lugar de leerse desde la configuracion.</li>
<li><strong>Acceso env fuera de config</strong> — Llamadas a <code>os.environ.get()</code>, <code>process.env.</code>, <code>getenv()</code> o <code>$_ENV</code> fuera del directorio de configuracion.</li>
</ul>

<h2 id="framework-awareness">Filtrado consciente del framework</h2>
<p>AigisCode aplica filtrado inteligente: los archivos de prueba se omiten, los directorios de configuracion se excluyen de las advertencias de acceso env, y las convenciones del framework se respetan. Configure <code>skip_path_patterns</code> en su politica.</p>

<h2 id="fixing-hardwiring">Corregir valores hardcoded</h2>
<ul>
<li><strong>Literales repetidos</strong> — Extraer a una constante nombrada en un modulo compartido.</li>
<li><strong>URLs/IPs hardcodeadas</strong> — Mover a variables de entorno o un archivo de configuracion.</li>
<li><strong>Acceso env disperso</strong> — Centralizar todas las lecturas de <code>os.environ</code> / <code>process.env</code> en un solo modulo de configuracion.</li>
<li><strong>Cadenas magicas</strong> — Reemplazar con enums o constantes nombradas.</li>
</ul>`,
      zh: `<h2 id="what-is-hardwiring">什么是硬编码？</h2>
<p>硬编码是将配置值直接嵌入应用程序代码而不是集中在配置文件或环境变量中的做法。数据库URL硬编码在服务类中，API端点在五个文件中重复，IP地址埋在中间件函数中——这些都是造成维护噩梦的硬编码值。</p>

<h2 id="four-categories">四类硬编码</h2>
<ul>
<li><strong>魔法字符串</strong> — 编码业务逻辑或状态值的字符串字面量，分散在各文件中。</li>
<li><strong>重复字面量</strong> — 同一字面值在不同目录中出现3次或更多。阈值可通过 <code>repeated_literal_min_occurrences</code> 配置。</li>
<li><strong>硬编码IP和URL</strong> — 嵌入代码中的IP地址、URL和连接字符串，而非从配置中读取。</li>
<li><strong>配置外的环境访问</strong> — 在配置目录之外调用 <code>os.environ.get()</code>、<code>process.env.</code>、<code>getenv()</code> 或 <code>$_ENV</code>。</li>
</ul>

<h2 id="framework-awareness">框架感知过滤</h2>
<p>AigisCode应用智能过滤：测试文件被跳过，配置目录排除在环境访问警告之外，框架约定被尊重。在策略中配置 <code>skip_path_patterns</code>。</p>

<h2 id="fixing-hardwiring">修复硬编码值</h2>
<ul>
<li><strong>重复字面量</strong> — 提取到共享模块中的命名常量。</li>
<li><strong>硬编码URL/IP</strong> — 移至环境变量或配置文件。</li>
<li><strong>分散的env访问</strong> — 将所有 <code>os.environ</code> / <code>process.env</code> 读取集中到单一配置模块。</li>
<li><strong>魔法字符串</strong> — 用枚举或命名常量替换。</li>
</ul>`,
      hi: `<h2 id="what-is-hardwiring">हार्डवायरिंग क्या है?</h2>
<p>हार्डवायरिंग कॉन्फ़िगरेशन वैल्यू को कॉन्फ़िगरेशन फ़ाइलों या एनवायरनमेंट वेरिएबल में केंद्रीकृत करने के बजाय सीधे एप्लिकेशन कोड में एम्बेड करने की प्रथा है। एक सर्विस क्लास में हार्डकोडेड डेटाबेस URL, पांच फ़ाइलों में दोहराया गया API एंडपॉइंट, एक मिडलवेयर फ़ंक्शन में दबी IP एड्रेस — ये सब हार्डवायर्ड वैल्यू हैं जो मेंटेनेंस के लिए बुरे सपने बनाती हैं।</p>

<h2 id="four-categories">हार्डवायरिंग की चार श्रेणियां</h2>
<ul>
<li><strong>मैजिक स्ट्रिंग्स</strong> — बिज़नेस लॉजिक या स्टेटस वैल्यू एनकोड करने वाले स्ट्रिंग लिटरल जो फ़ाइलों में बिखरे होते हैं।</li>
<li><strong>दोहराए गए लिटरल</strong> — एक ही लिटरल वैल्यू विभिन्न डायरेक्टरी में 3 या अधिक बार दिखाई देना। थ्रेशोल्ड <code>repeated_literal_min_occurrences</code> के माध्यम से कॉन्फ़िगर किया जा सकता है।</li>
<li><strong>हार्डकोडेड IP और URL</strong> — कॉन्फ़िगरेशन से पढ़ने के बजाय कोड में एम्बेड किए गए IP एड्रेस, URL और कनेक्शन स्ट्रिंग।</li>
<li><strong>कॉन्फ़िग के बाहर env एक्सेस</strong> — कॉन्फ़िगरेशन डायरेक्टरी के बाहर <code>os.environ.get()</code>, <code>process.env.</code>, <code>getenv()</code> या <code>$_ENV</code> की कॉल।</li>
</ul>

<h2 id="framework-awareness">फ्रेमवर्क-अवेयर फ़िल्टरिंग</h2>
<p>AigisCode इंटेलिजेंट फ़िल्टरिंग लागू करता है: टेस्ट फ़ाइलें स्किप की जाती हैं, कॉन्फ़िगरेशन डायरेक्टरी env-access चेतावनियों से बाहर रखी जाती हैं, और फ्रेमवर्क कन्वेंशन का सम्मान किया जाता है। अपनी पॉलिसी में <code>skip_path_patterns</code> कॉन्फ़िगर करें।</p>

<h2 id="fixing-hardwiring">हार्डवायर्ड वैल्यू ठीक करना</h2>
<ul>
<li><strong>दोहराए गए लिटरल</strong> — शेयर्ड मॉड्यूल में नामित कॉन्स्टेंट में एक्सट्रैक्ट करें।</li>
<li><strong>हार्डकोडेड URL/IP</strong> — एनवायरनमेंट वेरिएबल या कॉन्फ़िगरेशन फ़ाइल में ले जाएं।</li>
<li><strong>बिखरा env एक्सेस</strong> — सभी <code>os.environ</code> / <code>process.env</code> रीड को एक कॉन्फ़िगरेशन मॉड्यूल में केंद्रीकृत करें।</li>
<li><strong>मैजिक स्ट्रिंग्स</strong> — enums या नामित कॉन्स्टेंट से बदलें।</li>
</ul>`,
      pt: `<h2 id="what-is-hardwiring">O que e hardwiring?</h2>
<p>Hardwiring e a pratica de incorporar valores de configuracao diretamente no codigo da aplicacao em vez de centraliza-los em arquivos de configuracao ou variaveis de ambiente. Uma URL de banco de dados hardcoded em uma classe de servico, um endpoint de API repetido em cinco arquivos, um endereco IP enterrado em uma funcao de middleware — todos sao valores hardcoded que criam pesadelos de manutencao.</p>

<h2 id="four-categories">Quatro categorias de hardwiring</h2>
<ul>
<li><strong>Strings magicas</strong> — Literais de string que codificam logica de negocios ou valores de estado espalhados pelos arquivos.</li>
<li><strong>Literais repetidos</strong> — O mesmo valor literal aparecendo 3 ou mais vezes em diretorios diferentes. O limite e configuravel via <code>repeated_literal_min_occurrences</code>.</li>
<li><strong>IPs e URLs hardcoded</strong> — Enderecos IP, URLs e strings de conexao incorporados no codigo em vez de lidos da configuracao.</li>
<li><strong>Acesso env fora do config</strong> — Chamadas a <code>os.environ.get()</code>, <code>process.env.</code>, <code>getenv()</code> ou <code>$_ENV</code> fora do diretorio de configuracao.</li>
</ul>

<h2 id="framework-awareness">Filtragem adaptada a frameworks</h2>
<p>AigisCode aplica filtragem inteligente: arquivos de teste sao ignorados, diretorios de configuracao sao excluidos dos avisos de acesso env, e convencoes de framework sao respeitadas. Configure <code>skip_path_patterns</code> na sua politica.</p>

<h2 id="fixing-hardwiring">Corrigir valores hardcoded</h2>
<ul>
<li><strong>Literais repetidos</strong> — Extrair para uma constante nomeada em um modulo compartilhado.</li>
<li><strong>URLs/IPs hardcoded</strong> — Mover para variaveis de ambiente ou arquivo de configuracao.</li>
<li><strong>Acesso env disperso</strong> — Centralizar todas as leituras de <code>os.environ</code> / <code>process.env</code> em um unico modulo de configuracao.</li>
<li><strong>Strings magicas</strong> — Substituir por enums ou constantes nomeadas.</li>
</ul>`,
      ar: `<h2 id="what-is-hardwiring">ما هي القيم الثابتة؟</h2>
<p>القيم الثابتة هي ممارسة تضمين قيم التكوين مباشرة في شيفرة التطبيق بدلاً من تجميعها في ملفات تكوين أو متغيرات بيئة. عنوان URL لقاعدة بيانات مشفّر في فئة خدمة أو نقطة نهاية API متكررة في خمسة ملفات أو عنوان IP مدفون في دالة وسيط — كلها قيم ثابتة تخلق كوابيس صيانة.</p>

<h2 id="four-categories">أربع فئات من القيم الثابتة</h2>
<ul>
<li><strong>السلاسل السحرية</strong> — قيم نصية حرفية تشفّر منطق العمل أو قيم الحالة مبعثرة عبر الملفات.</li>
<li><strong>القيم الحرفية المتكررة</strong> — نفس القيمة الحرفية تظهر ٣ مرات أو أكثر في مجلدات مختلفة. الحد قابل للتكوين عبر <code>repeated_literal_min_occurrences</code>.</li>
<li><strong>عناوين IP وURLs المشفّرة</strong> — عناوين IP وURLs وسلاسل الاتصال المضمّنة في الشيفرة بدلاً من قراءتها من التكوين.</li>
<li><strong>وصول متغيرات البيئة خارج التكوين</strong> — استدعاءات <code>os.environ.get()</code> و<code>process.env.</code> و<code>getenv()</code> أو <code>$_ENV</code> خارج مجلد التكوين.</li>
</ul>

<h2 id="framework-awareness">تصفية واعية بالإطار</h2>
<p>يطبّق AigisCode تصفية ذكية: تُتجاهل ملفات الاختبار وتُستبعد مجلدات التكوين من تحذيرات وصول متغيرات البيئة وتُحترم اتفاقيات الإطار. كوّن <code>skip_path_patterns</code> في سياستك.</p>

<h2 id="fixing-hardwiring">إصلاح القيم الثابتة</h2>
<ul>
<li><strong>القيم الحرفية المتكررة</strong> — استخرجها إلى ثابت مُسمى في وحدة مشتركة.</li>
<li><strong>عناوين URLs/IPs المشفّرة</strong> — انقلها إلى متغيرات بيئة أو ملف تكوين.</li>
<li><strong>وصول متغيرات البيئة المبعثر</strong> — اجمع كل قراءات <code>os.environ</code> / <code>process.env</code> في وحدة تكوين واحدة.</li>
<li><strong>السلاسل السحرية</strong> — استبدلها بتعدادات أو ثوابت مُسماة.</li>
</ul>`,
      pl: `<h2 id="what-is-hardwiring">Czym sa zakodowane wartosci?</h2>
<p>Detektor AigisCode identyfikuje magiczne stringi, powtarzajace sie literaly, zakodowane adresy sieciowe i dostep do zmiennych srodowiskowych poza warstwa konfiguracyjna. Respektuje wylaczenia polityk dla wzorow frameworkowych.</p>`,
    },
    capabilities: [
      'Magic string detection',
      'Repeated literal detection (configurable threshold)',
      'Hardcoded IP and URL detection',
      'Environment access outside config detection',
      'Cross-directory literal aggregation',
      'Framework-aware filtering',
      'Configurable skip path patterns',
      'Policy-driven thresholds',
    ],
    codeExample: `# Analyze for hardwired values
aigiscode analyze /path/to/project

# View all hardwiring findings
cat .aigiscode/deterministic-analysis.json | jq '.hardwiring'

# Configure detection thresholds
cat > .aigiscode/policy.json << EOF
{
  "hardwiring": {
    "repeated_literal_min_occurrences": 4,
    "skip_path_patterns": ["tests/**/*"]
  }
}
EOF`,
    relatedSlugs: ['dead-code', 'ai-review', 'layer-violations'],
  },
  /* ---- 4. AI-Assisted Review ---- */
  {
    slug: 'ai-review',
    icon: 'Brain',
    category: 'AI-Powered',
    title: {
      en: 'AI-Assisted Review',
      cs: 'Revize s pomocí AI',
      fr: 'Revue assistée par IA',
      es: 'Revisión asistida por IA',
      zh: 'AI 辅助审查',
      hi: 'AI-सहायता प्राप्त समीक्षा',
      pt: 'Revisão assistida por IA',
      ar: 'مراجعة بمساعدة الذكاء الاصطناعي',
      pl: 'Przeglad wspomagany AI',
      bn: 'AI-সহায়তা পর্যালোচনা',
    },
    shortDescription: {
      en: 'AI classifies findings as true positives, false positives, or needs-context. Proposes structural exclusion rules to reduce noise across future runs. Supports OpenAI and Anthropic backends.',
      cs: 'AI klasifikuje nálezy jako skutečné pozitivy, falešné pozitivy nebo vyžadující kontext. Navrhuje strukturální pravidla vyloučení pro snížení šumu v budoucích analýzách. Podporuje backendy OpenAI a Anthropic.',
      fr: 'L\'IA classe les résultats en vrais positifs, faux positifs ou nécessitant un contexte. Propose des règles d\'exclusion structurelles pour réduire le bruit dans les analyses futures. Compatible avec les backends OpenAI et Anthropic.',
      es: 'La IA clasifica los hallazgos como verdaderos positivos, falsos positivos o que necesitan contexto. Propone reglas de exclusión estructurales para reducir el ruido en ejecuciones futuras. Compatible con backends OpenAI y Anthropic.',
      zh: 'AI 将发现分类为真正问题、误报或需要上下文。提出结构化排除规则以减少未来运行中的噪音。支持 OpenAI 和 Anthropic 后端。',
      hi: 'AI निष्कर्षों को ट्रू पॉज़िटिव, फ़ॉल्स पॉज़िटिव या कॉन्टेक्स्ट-ज़रूरी के रूप में वर्गीकृत करता है। भविष्य के रन में शोर कम करने के लिए संरचनात्मक एक्सक्लूज़न नियम प्रस्तावित करता है। OpenAI और Anthropic बैकएंड को सपोर्ट करता है।',
      pt: 'A IA classifica as descobertas como verdadeiros positivos, falsos positivos ou que necessitam de contexto. Propõe regras de exclusão estruturais para reduzir ruído em execuções futuras. Compatível com backends OpenAI e Anthropic.',
      ar: 'يصنّف الذكاء الاصطناعي النتائج إلى إيجابيات حقيقية أو إيجابيات كاذبة أو تحتاج سياقاً. يقترح قواعد استبعاد هيكلية لتقليل الضوضاء في التشغيلات المستقبلية. يدعم واجهات OpenAI وAnthropic.',
      pl: 'AI klasyfikuje znaleziska jako prawdziwe problemy, falszywe alarmy lub wymagajace kontekstu. Proponuje strukturalne reguly wykluczajace, ktore utrzymuja sie miedzy uruchomieniami. Obsluguje backendy OpenAI i Anthropic.',
      bn: 'AI ফলাফলকে ট্রু পজিটিভ, ফলস পজিটিভ বা প্রসঙ্গ-নির্ভর হিসেবে শ্রেণিবদ্ধ করে। ভবিষ্যতের রানে শব্দ কমাতে কাঠামোগত বর্জন নিয়ম প্রস্তাব করে। OpenAI এবং Anthropic ব্যাকএন্ড সাপোর্ট করে।',
    },
    metaDescription: {
      en: 'AI-assisted code review that classifies static analysis findings using OpenAI or Anthropic. AigisCode AI Review reduces false positives and proposes exclusion rules automatically.',
      cs: 'Revize kódu s pomocí AI, která klasifikuje nálezy statické analýzy pomocí OpenAI nebo Anthropic. AigisCode AI Review snižuje falešné pozitivy a automaticky navrhuje pravidla vyloučení.',
      fr: 'Revue de code assistée par IA qui classe les résultats d\'analyse statique avec OpenAI ou Anthropic. AigisCode AI Review réduit les faux positifs et propose automatiquement des règles d\'exclusion.',
      es: 'Revisión de código asistida por IA que clasifica hallazgos de análisis estático usando OpenAI o Anthropic. AigisCode AI Review reduce falsos positivos y propone reglas de exclusión automáticamente.',
      zh: '使用 OpenAI 或 Anthropic 对静态分析发现进行分类的 AI 辅助代码审查。AigisCode AI Review 减少误报并自动提出排除规则。',
      hi: 'AI-सहायता प्राप्त कोड समीक्षा जो OpenAI या Anthropic का उपयोग करके स्टैटिक एनालिसिस निष्कर्षों को वर्गीकृत करती है। AigisCode AI Review फ़ॉल्स पॉज़िटिव को कम करता है और स्वचालित रूप से एक्सक्लूज़न नियम प्रस्तावित करता है।',
      pt: 'Revisão de código assistida por IA que classifica descobertas de análise estática usando OpenAI ou Anthropic. AigisCode AI Review reduz falsos positivos e propõe regras de exclusão automaticamente.',
      ar: 'مراجعة شيفرة بمساعدة الذكاء الاصطناعي تصنّف نتائج التحليل الثابت باستخدام OpenAI أو Anthropic. تقلل مراجعة AigisCode بالذكاء الاصطناعي الإيجابيات الكاذبة وتقترح قواعد استبعاد تلقائياً.',
      pl: 'Przeglad kodu wspomagany AI, ktory klasyfikuje znaleziska analizy statycznej za pomoca OpenAI lub Anthropic. AigisCode redukuje falszywe alarmy i proponuje reguly wykluczajace dla szybszych i dokladniejszych przegladow.',
      bn: 'OpenAI বা Anthropic ব্যবহার করে স্ট্যাটিক অ্যানালিসিস ফলাফল শ্রেণিবদ্ধ করে AI-সহায়তা কোড রিভিউ। AigisCode AI রিভিউ ফলস পজিটিভ কমায় এবং স্বয়ংক্রিয়ভাবে বর্জন নিয়ম প্রস্তাব করে।',
    },
    content: {
      en: `<h2 id="why-ai-review">Why AI-Assisted Review?</h2>
<p>Static analysis produces findings. Some are real problems; others are false positives caused by framework conventions, metaprogramming, or patterns the detector cannot fully understand. Manually triaging hundreds of findings is slow and error-prone. AigisCode's AI Review stage uses large language models to classify each finding, dramatically reducing the noise-to-signal ratio.</p>
<p>AI Review is the <strong>fifth stage</strong> of AigisCode's six-stage pipeline. It runs after the deterministic detectors (Index, Graph, Detect, Rules) have produced their findings. The AI does not replace the detectors — it classifies their output. This design means that every finding has a deterministic origin that can be traced, and the AI layer adds judgment without obscuring the evidence.</p>

<h2 id="classification-output">Classification Output</h2>
<p>For each finding, the AI reviewer produces a classification:</p>
<ul>
<li><strong>True positive</strong> — The finding is a real issue that should be addressed. The AI provides reasoning for why the code is problematic.</li>
<li><strong>False positive</strong> — The finding is not a real issue. The AI explains why the detector flagged it and why it is actually safe (e.g., framework convention, dynamic dispatch).</li>
<li><strong>Needs context</strong> — The AI cannot determine the classification without additional information. These findings require human review.</li>
</ul>
<p>This three-way classification lets teams focus their time on true positives and uncertain findings, while confidently dismissing false positives.</p>

<h2 id="rule-proposals">Automatic Rule Proposals</h2>
<p>When the AI classifies a finding as a false positive, it can propose a <strong>structural exclusion rule</strong> that will suppress similar findings in future runs. AigisCode supports 8 rule types:</p>
<ul>
<li><strong>Path-based rules</strong> — Suppress findings in specific files or directories</li>
<li><strong>Symbol-based rules</strong> — Suppress findings for specific class names, methods, or functions</li>
<li><strong>Pattern-based rules</strong> — Suppress findings matching a regex pattern in the finding description</li>
<li><strong>Type-based rules</strong> — Suppress all findings of a specific detector type</li>
<li><strong>Category-based rules</strong> — Suppress findings by category (e.g., all dead code in migration files)</li>
<li><strong>Confidence-based rules</strong> — Suppress findings below a confidence threshold</li>
<li><strong>Compound rules</strong> — Combine multiple conditions with AND/OR logic</li>
<li><strong>Scope-based rules</strong> — Suppress findings at project, directory, or file scope</li>
</ul>
<p>Proposed rules are written to <code>.aigiscode/rules.json</code> where they can be reviewed, edited, and committed to version control. This creates a learning loop: each analysis run refines the rule set, reducing false positives over time without modifying the detectors.</p>

<h2 id="backends">Supported AI Backends</h2>
<p>AigisCode supports two AI backend providers:</p>
<ul>
<li><strong>OpenAI</strong> — GPT-4o and GPT-4 Turbo for high-accuracy classification. Configure with <code>OPENAI_API_KEY</code>.</li>
<li><strong>Anthropic</strong> — Claude 3.5 Sonnet and Claude 3 Opus for nuanced reasoning about code patterns. Configure with <code>ANTHROPIC_API_KEY</code>.</li>
</ul>
<p>Backend selection is automatic based on available API keys, or can be specified explicitly. The AI reviewer sends the finding context (file path, code snippet, detector type, confidence level) to the model and parses the structured response.</p>

<h2 id="deterministic-mode">Running Without AI</h2>
<p>AI Review is optional. Run AigisCode with <code>--skip-ai</code> for fully deterministic analysis with zero API costs. This is recommended for CI/CD pipelines where reproducibility is critical. The deterministic detectors provide all the structural findings; AI Review is an accuracy refinement layer, not a requirement.</p>

<h2 id="design-principles">Design Principles</h2>
<p>AigisCode treats AI review as classification, not detection:</p>
<ul>
<li>The AI never invents findings — it only classifies what the deterministic detectors produce.</li>
<li>Every classification includes reasoning that a human can verify.</li>
<li>Rule proposals are suggestions, not automatic suppressions. A human must approve them.</li>
<li>The AI reviewer does not mutate detector behavior. It operates as a read-only triage layer.</li>
</ul>`,
      cs: `<h2 id="why-ai-review">Proč AI revize?</h2>
<p>Statická analýza produkuje nálezy. Některé jsou skutečné problémy; jiné jsou falešné pozitivy způsobené konvencemi frameworku nebo metaprogramováním. AigisCode AI Review používá velké jazykové modely ke klasifikaci každého nálezu, což dramaticky snižuje poměr šumu k signálu.</p>
<p>AI Review je <strong>pátá fáze</strong> šestifázového pipeline AigisCode. Běží po deterministických detektorech (Index, Graf, Detekce, Pravidla). AI nenahrazuje detektory — klasifikuje jejich výstup.</p>

<h2 id="classification-output">Výstup klasifikace</h2>
<ul>
<li><strong>True positive</strong> — Nález je skutečný problém, který by měl být vyřešen.</li>
<li><strong>False positive</strong> — Nález není skutečný problém. AI vysvětluje proč.</li>
<li><strong>Needs context</strong> — AI nemůže určit klasifikaci bez dalších informací.</li>
</ul>

<h2 id="rule-proposals">Automatické návrhy pravidel</h2>
<p>Když AI klasifikuje nález jako false positive, může navrhnout strukturální pravidlo vyloučení. AigisCode podporuje 8 typů pravidel: na základě cesty, symbolu, vzoru, typu, kategorie, důvěryhodnosti, složená pravidla a pravidla na základě rozsahu. Pravidla se zapisují do <code>.aigiscode/rules.json</code>.</p>

<h2 id="backends">Podporované AI backendy</h2>
<ul>
<li><strong>OpenAI</strong> — GPT-4o a GPT-4 Turbo. Konfigurujte pomocí <code>OPENAI_API_KEY</code>.</li>
<li><strong>Anthropic</strong> — Claude 3.5 Sonnet a Claude 3 Opus. Konfigurujte pomocí <code>ANTHROPIC_API_KEY</code>.</li>
</ul>

<h2 id="deterministic-mode">Provoz bez AI</h2>
<p>AI Review je volitelná. Spusťte AigisCode s <code>--skip-ai</code> pro plně deterministickou analýzu s nulovými náklady na API. Doporučeno pro CI/CD pipeline, kde je klíčová reprodukovatelnost.</p>`,
      fr: `<h2 id="why-ai-review">Pourquoi la revue par IA ?</h2>
<p>L'analyse statique produit des resultats. Certains sont de vrais problemes ; d'autres sont des faux positifs causes par les conventions de framework ou la metaprogrammation. L'etape AI Review d'AigisCode utilise des grands modeles de langage pour classifier chaque resultat, reduisant considerablement le rapport bruit/signal.</p>
<p>AI Review est la <strong>cinquieme etape</strong> du pipeline en six etapes d'AigisCode. Elle s'execute apres les detecteurs deterministes (Index, Graphe, Detection, Regles). L'IA ne remplace pas les detecteurs — elle classifie leur sortie.</p>

<h2 id="classification-output">Sortie de classification</h2>
<ul>
<li><strong>Vrai positif</strong> — Le resultat est un vrai probleme a traiter.</li>
<li><strong>Faux positif</strong> — Le resultat n'est pas un vrai probleme. L'IA explique pourquoi.</li>
<li><strong>Necessite du contexte</strong> — L'IA ne peut pas determiner la classification sans informations supplementaires.</li>
</ul>

<h2 id="rule-proposals">Propositions automatiques de regles</h2>
<p>Lorsque l'IA classifie un resultat comme faux positif, elle peut proposer une regle d'exclusion structurelle. AigisCode supporte 8 types de regles : basees sur le chemin, le symbole, le motif, le type, la categorie, la confiance, les regles composees et basees sur la portee. Les regles sont ecrites dans <code>.aigiscode/rules.json</code>.</p>

<h2 id="backends">Backends IA supportes</h2>
<ul>
<li><strong>OpenAI</strong> — GPT-4o et GPT-4 Turbo. Configurer avec <code>OPENAI_API_KEY</code>.</li>
<li><strong>Anthropic</strong> — Claude 3.5 Sonnet et Claude 3 Opus. Configurer avec <code>ANTHROPIC_API_KEY</code>.</li>
</ul>

<h2 id="deterministic-mode">Execution sans IA</h2>
<p>AI Review est optionnelle. Executez AigisCode avec <code>--skip-ai</code> pour une analyse purement deterministe sans cout d'API. Recommande pour les pipelines CI/CD ou la reproductibilite est essentielle.</p>`,
      es: `<h2 id="why-ai-review">Por que revision asistida por IA?</h2>
<p>El analisis estatico produce hallazgos. Algunos son problemas reales; otros son falsos positivos causados por convenciones de framework o metaprogramacion. La etapa AI Review de AigisCode usa grandes modelos de lenguaje para clasificar cada hallazgo, reduciendo dramaticamente la relacion ruido/senal.</p>
<p>AI Review es la <strong>quinta etapa</strong> del pipeline de seis etapas de AigisCode. Se ejecuta despues de los detectores deterministicos (Index, Grafo, Deteccion, Reglas). La IA no reemplaza los detectores — clasifica su salida.</p>

<h2 id="classification-output">Salida de clasificacion</h2>
<ul>
<li><strong>Verdadero positivo</strong> — El hallazgo es un problema real que debe abordarse.</li>
<li><strong>Falso positivo</strong> — El hallazgo no es un problema real. La IA explica por que.</li>
<li><strong>Necesita contexto</strong> — La IA no puede determinar la clasificacion sin informacion adicional.</li>
</ul>

<h2 id="rule-proposals">Propuestas automaticas de reglas</h2>
<p>Cuando la IA clasifica un hallazgo como falso positivo, puede proponer una regla de exclusion estructural. AigisCode soporta 8 tipos de reglas: basadas en ruta, simbolo, patron, tipo, categoria, confianza, reglas compuestas y basadas en alcance. Las reglas se escriben en <code>.aigiscode/rules.json</code>.</p>

<h2 id="backends">Backends de IA soportados</h2>
<ul>
<li><strong>OpenAI</strong> — GPT-4o y GPT-4 Turbo. Configure con <code>OPENAI_API_KEY</code>.</li>
<li><strong>Anthropic</strong> — Claude 3.5 Sonnet y Claude 3 Opus. Configure con <code>ANTHROPIC_API_KEY</code>.</li>
</ul>

<h2 id="deterministic-mode">Ejecucion sin IA</h2>
<p>AI Review es opcional. Ejecute AigisCode con <code>--skip-ai</code> para analisis puramente deterministico sin costos de API. Recomendado para pipelines CI/CD donde la reproducibilidad es critica.</p>`,
      zh: `<h2 id="why-ai-review">为什么需要AI辅助审查？</h2>
<p>静态分析会产生发现。有些是真正的问题；其他是由框架约定或元编程引起的误报。AigisCode的AI Review阶段使用大型语言模型对每个发现进行分类，大幅降低噪声信号比。</p>
<p>AI Review是AigisCode六阶段流水线的<strong>第五阶段</strong>。它在确定性检测器（索引、图构建、检测、规则）之后运行。AI不替代检测器——它对检测器的输出进行分类。</p>

<h2 id="classification-output">分类输出</h2>
<ul>
<li><strong>真阳性</strong> — 发现是需要解决的真实问题。</li>
<li><strong>假阳性</strong> — 发现不是真实问题。AI解释原因。</li>
<li><strong>需要上下文</strong> — AI无法在没有额外信息的情况下确定分类。</li>
</ul>

<h2 id="rule-proposals">自动规则提案</h2>
<p>当AI将发现分类为假阳性时，它可以提出结构化排除规则。AigisCode支持8种规则类型：基于路径、符号、模式、类型、类别、置信度、复合规则和基于范围的规则。规则写入 <code>.aigiscode/rules.json</code>。</p>

<h2 id="backends">支持的AI后端</h2>
<ul>
<li><strong>OpenAI</strong> — GPT-4o和GPT-4 Turbo。通过 <code>OPENAI_API_KEY</code> 配置。</li>
<li><strong>Anthropic</strong> — Claude 3.5 Sonnet和Claude 3 Opus。通过 <code>ANTHROPIC_API_KEY</code> 配置。</li>
</ul>

<h2 id="deterministic-mode">无AI运行</h2>
<p>AI Review是可选的。使用 <code>--skip-ai</code> 运行AigisCode进行完全确定性分析，零API成本。建议用于需要可重现性的CI/CD流水线。</p>`,
      hi: `<h2 id="why-ai-review">AI-सहायित समीक्षा क्यों?</h2>
<p>स्टैटिक एनालिसिस फ़ाइंडिंग्स प्रोड्यूस करता है। कुछ वास्तविक समस्याएं होती हैं; अन्य फ्रेमवर्क कन्वेंशन या मेटाप्रोग्रामिंग के कारण फ़ॉल्स पॉज़िटिव होती हैं। AigisCode का AI Review स्टेज हर फ़ाइंडिंग को क्लासिफाई करने के लिए लार्ज लैंग्वेज मॉडल का उपयोग करता है, जिससे नॉइज़-टू-सिग्नल रेशियो नाटकीय रूप से कम होता है।</p>
<p>AI Review AigisCode के छह-चरण पाइपलाइन का <strong>पांचवां चरण</strong> है। यह डिटर्मिनिस्टिक डिटेक्टर्स (Index, Graph, Detect, Rules) के बाद चलता है। AI डिटेक्टर्स को रिप्लेस नहीं करता — यह उनके आउटपुट को क्लासिफाई करता है।</p>

<h2 id="classification-output">क्लासिफिकेशन आउटपुट</h2>
<ul>
<li><strong>ट्रू पॉज़िटिव</strong> — फ़ाइंडिंग एक वास्तविक समस्या है जिसे संबोधित किया जाना चाहिए।</li>
<li><strong>फ़ॉल्स पॉज़िटिव</strong> — फ़ाइंडिंग वास्तविक समस्या नहीं है। AI बताता है क्यों।</li>
<li><strong>कॉन्टेक्स्ट चाहिए</strong> — AI अतिरिक्त जानकारी के बिना क्लासिफिकेशन निर्धारित नहीं कर सकता।</li>
</ul>

<h2 id="rule-proposals">स्वचालित नियम प्रस्ताव</h2>
<p>जब AI किसी फ़ाइंडिंग को फ़ॉल्स पॉज़िटिव के रूप में क्लासिफाई करता है, तो वह एक स्ट्रक्चरल एक्सक्लूज़न रूल प्रस्तावित कर सकता है। AigisCode 8 प्रकार के नियमों का समर्थन करता है: पाथ-बेस्ड, सिंबल-बेस्ड, पैटर्न-बेस्ड, टाइप-बेस्ड, कैटेगरी-बेस्ड, कॉन्फिडेंस-बेस्ड, कंपाउंड और स्कोप-बेस्ड। नियम <code>.aigiscode/rules.json</code> में लिखे जाते हैं।</p>

<h2 id="backends">समर्थित AI बैकएंड</h2>
<ul>
<li><strong>OpenAI</strong> — GPT-4o और GPT-4 Turbo। <code>OPENAI_API_KEY</code> से कॉन्फ़िगर करें।</li>
<li><strong>Anthropic</strong> — Claude 3.5 Sonnet और Claude 3 Opus। <code>ANTHROPIC_API_KEY</code> से कॉन्फ़िगर करें।</li>
</ul>

<h2 id="deterministic-mode">AI के बिना चलाना</h2>
<p>AI Review वैकल्पिक है। शून्य API लागत के साथ पूर्णतः डिटर्मिनिस्टिक एनालिसिस के लिए AigisCode को <code>--skip-ai</code> के साथ चलाएं। CI/CD पाइपलाइन के लिए अनुशंसित जहां रिप्रोड्यूसिबिलिटी महत्वपूर्ण है।</p>`,
      pt: `<h2 id="why-ai-review">Por que revisao assistida por IA?</h2>
<p>A analise estatica produz descobertas. Algumas sao problemas reais; outras sao falsos positivos causados por convencoes de framework ou metaprogramacao. A etapa AI Review do AigisCode usa grandes modelos de linguagem para classificar cada descoberta, reduzindo drasticamente a relacao ruido/sinal.</p>
<p>AI Review e a <strong>quinta etapa</strong> do pipeline de seis etapas do AigisCode. Executa apos os detectores deterministicos (Index, Grafo, Deteccao, Regras). A IA nao substitui os detectores — classifica sua saida.</p>

<h2 id="classification-output">Saida de classificacao</h2>
<ul>
<li><strong>Verdadeiro positivo</strong> — A descoberta e um problema real que deve ser abordado.</li>
<li><strong>Falso positivo</strong> — A descoberta nao e um problema real. A IA explica por que.</li>
<li><strong>Necessita contexto</strong> — A IA nao pode determinar a classificacao sem informacoes adicionais.</li>
</ul>

<h2 id="rule-proposals">Propostas automaticas de regras</h2>
<p>Quando a IA classifica uma descoberta como falso positivo, pode propor uma regra de exclusao estrutural. AigisCode suporta 8 tipos de regras: baseadas em caminho, simbolo, padrao, tipo, categoria, confianca, regras compostas e baseadas em escopo. As regras sao escritas em <code>.aigiscode/rules.json</code>.</p>

<h2 id="backends">Backends de IA suportados</h2>
<ul>
<li><strong>OpenAI</strong> — GPT-4o e GPT-4 Turbo. Configure com <code>OPENAI_API_KEY</code>.</li>
<li><strong>Anthropic</strong> — Claude 3.5 Sonnet e Claude 3 Opus. Configure com <code>ANTHROPIC_API_KEY</code>.</li>
</ul>

<h2 id="deterministic-mode">Execucao sem IA</h2>
<p>AI Review e opcional. Execute AigisCode com <code>--skip-ai</code> para analise puramente deterministica sem custos de API. Recomendado para pipelines CI/CD onde a reprodutibilidade e critica.</p>`,
      ar: `<h2 id="false-positive-problem">مشكلة الإيجابيات الكاذبة</h2>
<p>ينتج التحليل الثابت نتائج. بعضها مشكلات حقيقية وبعضها إيجابيات كاذبة ناتجة عن اتفاقيات الإطار أو البرمجة الوصفية. يؤتمت AigisCode الفرز باستخدام نماذج اللغة الكبيرة.</p>

<h2 id="how-it-works">كيف تعمل مراجعة الذكاء الاصطناعي</h2>
<p>مراجعة الذكاء الاصطناعي هي <strong>المرحلة الخامسة</strong> من خط أنابيب AigisCode المكون من ست مراحل. تعمل بعد الكاشفات الحتمية وتصنّف كل نتيجة:</p>
<ul>
<li><strong>إيجابية حقيقية</strong> — مشكلة حقيقية تحتاج إصلاحاً.</li>
<li><strong>إيجابية كاذبة</strong> — اتفاقية إطار أو نمط مقصود.</li>
<li><strong>يحتاج سياقاً</strong> — غير مؤكد ويحتاج مراجعة بشرية.</li>
</ul>

<h2 id="exclusion-rules">قواعد الاستبعاد التلقائية</h2>
<p>عندما يصنّف الذكاء الاصطناعي نتيجة كإيجابية كاذبة يقترح <strong>قاعدة استبعاد هيكلية</strong> تُكتب في <code>.aigiscode/rules.json</code>. تستهدف القواعد الأنماط وليس الملفات الفردية مما يقمع إيجابيات كاذبة متعددة بقاعدة واحدة.</p>

<h2 id="backends">واجهات الذكاء الاصطناعي المدعومة</h2>
<ul>
<li><strong>OpenAI</strong> — يعمل مع نماذج GPT-4o والأحدث. كوّن عبر <code>OPENAI_API_KEY</code>.</li>
<li><strong>Anthropic</strong> — يعمل مع نماذج Claude. كوّن عبر <code>ANTHROPIC_API_KEY</code>.</li>
</ul>
<p>مراجعة الذكاء الاصطناعي اختيارية. شغّل مع <code>--skip-ai</code> للتحليل الحتمي بالكامل بدون استدعاءات API. هذا الوضع موصى به لخطوط أنابيب CI.</p>`,
      pl: `<h2 id="ai-review">Przeglad wspomagany AI</h2>
<p>AigisCode integruje backendy OpenAI i Anthropic do klasyfikacji znalezisk. Proponowane reguly wykluczajace sa zapisywane dla przyszlych uruchomien.</p>
<h2 id="skip-ai">Tryb --skip-ai</h2>
<p>W CI uzyj flagi <code>--skip-ai</code> dla deterministycznych wynikow bez wywolan API.</p>`,
    },
    capabilities: [
      'Three-way classification (true/false positive, needs-context)',
      'Automatic exclusion rule proposals',
      '8 structural rule types',
      'OpenAI backend support (GPT-4o, GPT-4 Turbo)',
      'Anthropic backend support (Claude 3.5 Sonnet, Claude 3 Opus)',
      'Deterministic fallback with --skip-ai',
      'Classification reasoning output',
      'Learning loop through saved rules',
    ],
    codeExample: `# Run full analysis with AI review
aigiscode analyze /path/to/project

# Run deterministic-only (no AI, no API cost)
aigiscode analyze /path/to/project

# View AI-classified findings
cat .aigiscode/deterministic-analysis.json | jq '.review'

# Review proposed exclusion rules
cat .aigiscode/rules.json`,
    relatedSlugs: ['dead-code', 'hardwiring', 'god-classes'],
  },
  /* ---- 5. God Class Detection ---- */
  {
    slug: 'god-classes',
    icon: 'Warning',
    category: 'Architecture',
    title: {
      en: 'God Class Detection',
      cs: 'Detekce god tříd',
      fr: 'Détection des classes dieu',
      es: 'Detección de clases dios',
      zh: '上帝类检测',
      hi: 'गॉड क्लास पहचान',
      pt: 'Detecção de god classes',
      ar: 'اكتشاف الفئات الضخمة',
      pl: 'Wykrywanie god klas',
      bn: 'গড ক্লাস ডিটেকশন',
    },
    shortDescription: {
      en: 'Identify classes with excessive responsibility — 15+ methods, high coupling, or extreme instability scores. God classes violate single-responsibility and make codebases harder to test, extend, and maintain.',
      cs: 'Identifikujte třídy s nadměrnou odpovědností — 15+ metod, vysoká vazba nebo extrémní skóre nestability. God třídy porušují princip jedné odpovědnosti a ztěžují testování, rozšiřování a údržbu kódu.',
      fr: 'Identifiez les classes avec une responsabilité excessive — plus de 15 méthodes, un couplage élevé ou des scores d\'instabilité extrêmes. Les classes dieu violent le principe de responsabilité unique et rendent le code plus difficile à tester, étendre et maintenir.',
      es: 'Identifica clases con responsabilidad excesiva — más de 15 métodos, alto acoplamiento o puntuaciones de inestabilidad extremas. Las clases dios violan la responsabilidad única y hacen que el código sea más difícil de probar, extender y mantener.',
      zh: '识别职责过多的类——15 个以上方法、高耦合度或极端不稳定性分数。上帝类违反单一职责原则，使代码库更难测试、扩展和维护。',
      hi: 'अत्यधिक जिम्मेदारी वाली क्लास की पहचान करें — 15+ मेथड, उच्च कपलिंग, या अत्यधिक अस्थिरता स्कोर। गॉड क्लास सिंगल-रिस्पॉन्सिबिलिटी का उल्लंघन करती हैं और कोडबेस को टेस्ट, एक्सटेंड और मेंटेन करना कठिन बनाती हैं।',
      pt: 'Identifique classes com responsabilidade excessiva — mais de 15 métodos, alto acoplamento ou pontuações de instabilidade extremas. God classes violam a responsabilidade única e tornam a base de código mais difícil de testar, estender e manter.',
      ar: 'حدد الفئات ذات المسؤولية المفرطة — ١٥+ طريقة أو اقتران عالٍ أو درجات عدم استقرار شديدة. تنتهك الفئات الضخمة مبدأ المسؤولية الواحدة وتجعل قواعد الشيفرة أصعب في الاختبار والتوسيع والصيانة.',
      pl: 'Identyfikuj klasy z nadmierna odpowiedzialnoscia — ponad 15 metod, wysokie powiazanie lub ekstremalnie niestabilne wskazniki. AigisCode oznacza hotspoty architektoniczne naruszajace zasade pojedynczej odpowiedzialnosci.',
      bn: 'অতিরিক্ত দায়িত্বসম্পন্ন ক্লাস চিহ্নিত করুন — ১৫+ মেথড, উচ্চ কাপলিং বা চরম অস্থিরতা স্কোর। গড ক্লাস একক-দায়িত্ব লঙ্ঘন করে এবং কোডবেস টেস্ট, সম্প্রসারণ ও রক্ষণাবেক্ষণ কঠিন করে।',
    },
    metaDescription: {
      en: 'Detect god classes in PHP, Python, TypeScript, and JavaScript codebases. AigisCode identifies classes with 15+ methods, high coupling, and extreme instability scores that violate single-responsibility principle.',
      cs: 'Detekujte god třídy v kódových základnách PHP, Python, TypeScript a JavaScript. AigisCode identifikuje třídy s 15+ metodami, vysokou vazbou a extrémním skóre nestability, které porušují princip jedné odpovědnosti.',
      fr: 'Détectez les classes dieu dans les bases de code PHP, Python, TypeScript et JavaScript. AigisCode identifie les classes avec plus de 15 méthodes, un couplage élevé et des scores d\'instabilité extrêmes qui violent le principe de responsabilité unique.',
      es: 'Detecta clases dios en bases de código PHP, Python, TypeScript y JavaScript. AigisCode identifica clases con más de 15 métodos, alto acoplamiento y puntuaciones de inestabilidad extremas que violan el principio de responsabilidad única.',
      zh: '检测 PHP、Python、TypeScript 和 JavaScript 代码库中的上帝类。AigisCode 识别具有 15 个以上方法、高耦合度和极端不稳定性分数的违反单一职责原则的类。',
      hi: 'PHP, Python, TypeScript और JavaScript कोडबेस में गॉड क्लास का पता लगाएं। AigisCode 15+ मेथड, उच्च कपलिंग और अत्यधिक अस्थिरता स्कोर वाली क्लास की पहचान करता है जो सिंगल-रिस्पॉन्सिबिलिटी सिद्धांत का उल्लंघन करती हैं।',
      pt: 'Detecte god classes em bases de código PHP, Python, TypeScript e JavaScript. AigisCode identifica classes com mais de 15 métodos, alto acoplamento e pontuações de instabilidade extremas que violam o princípio de responsabilidade única.',
      ar: 'اكتشف الفئات الضخمة في قواعد شيفرة PHP وPython وTypeScript وJavaScript. يحدد AigisCode الفئات ذات ١٥+ طريقة والاقتران العالي ودرجات عدم الاستقرار الشديدة التي تنتهك مبدأ المسؤولية الواحدة.',
      pl: 'Wykrywaj god klasy w bazach kodu PHP, Python, TypeScript i JavaScript. AigisCode identyfikuje klasy z nadmierna liczba metod, wysokim powiazaniem i ekstremalna niestabilnoscia, tworzace hotspoty architektoniczne.',
      bn: 'PHP, Python, TypeScript এবং JavaScript কোডবেসে গড ক্লাস শনাক্ত করুন। AigisCode ১৫+ মেথড, উচ্চ কাপলিং এবং চরম অস্থিরতা স্কোরসম্পন্ন ক্লাস চিহ্নিত করে যা একক-দায়িত্ব নীতি লঙ্ঘন করে।',
    },
    content: {
      en: `<h2 id="what-are-god-classes">What Are God Classes?</h2>
<p>A god class is a class that does too much. It has too many methods, too many dependencies, and too many responsibilities. God classes emerge naturally as projects grow — a service class starts with 5 methods, gains 3 more during a sprint, picks up utility functions during a refactoring, and before anyone notices, it has 40 methods and touches every part of the system.</p>
<p>God classes are one of the strongest predictors of bug density. Research consistently shows that classes with high method counts and high coupling are disproportionately likely to contain defects. They are also the hardest classes to test because mocking their many dependencies is painful, and the hardest to extend because any change risks breaking unrelated functionality.</p>

<h2 id="detection-criteria">Detection Criteria</h2>
<p>AigisCode identifies god classes using multiple signals:</p>
<ul>
<li><strong>Method count threshold</strong> — Classes with 15 or more methods are flagged. This threshold catches classes that have grown beyond a reasonable single responsibility. The threshold is based on industry research showing that classes above this size exhibit significantly higher defect rates.</li>
<li><strong>High method + dependency count</strong> — Classes with a large number of methods AND a large number of dependencies (imports, injected services) are stronger god class candidates. A class with 15 methods but only 2 dependencies might be acceptable (a utility class). A class with 15 methods and 12 dependencies is almost certainly doing too much.</li>
<li><strong>Instability scores (Ca/Ce)</strong> — AigisCode computes Robert C. Martin's instability metric for each class: <code>I = Ce / (Ca + Ce)</code>, where Ca is afferent coupling (incoming dependencies) and Ce is efferent coupling (outgoing dependencies). Classes with extreme instability scores (near 0 or near 1) combined with high method counts indicate architectural problems.</li>
</ul>

<h2 id="instability-metric">The Instability Metric</h2>
<p>Understanding the instability metric helps prioritize which god classes to refactor first:</p>
<ul>
<li><strong>I = 0 (maximally stable)</strong> — Many things depend on this class, but it depends on nothing. This is a foundation class. If it is also a god class, changes to it have maximum blast radius. Refactor this first.</li>
<li><strong>I = 1 (maximally unstable)</strong> — This class depends on many things, but nothing depends on it. It is a leaf. If it is a god class, it is risky but has limited blast radius.</li>
<li><strong>I = 0.5 (balanced)</strong> — Equal incoming and outgoing dependencies. This is the neutral position.</li>
</ul>
<p>A stable god class (low I) is the most dangerous combination: many dependents rely on it, and its excessive method count means frequent changes. Each change risks breaking everything that depends on it.</p>

<h2 id="reading-the-report">Reading the Report</h2>
<p>God class findings appear in the graph analysis section of the report:</p>
<pre><code>{
  "graph_analysis": {
    "god_classes": [
      {
        "file": "src/services/UserService.php",
        "class": "UserService",
        "method_count": 32,
        "dependency_count": 14,
        "instability": 0.35,
        "confidence": "high"
      }
    ]
  }
}</code></pre>

<h2 id="refactoring-strategies">Refactoring God Classes</h2>
<p>Breaking up a god class requires careful planning. Common strategies include:</p>
<ul>
<li><strong>Extract service classes</strong> — Group related methods into focused service classes. A <code>UserService</code> with 32 methods might decompose into <code>UserAuthService</code>, <code>UserProfileService</code>, <code>UserNotificationService</code>, and <code>UserSearchService</code>.</li>
<li><strong>Strategy pattern</strong> — When the god class has many methods that are variations of the same operation, extract them into strategy objects.</li>
<li><strong>Facade pattern</strong> — Keep the god class as a thin facade that delegates to smaller, focused classes. This preserves the existing API while distributing the implementation.</li>
<li><strong>Event-driven decomposition</strong> — Replace direct method calls with events. Instead of the god class handling notifications directly, it emits events that notification handlers process.</li>
</ul>`,
      cs: `<h2 id="what-are-god-classes">Co jsou god třídy?</h2>
<p>God třída je třída, která dělá příliš mnoho. Má příliš mnoho metod, příliš mnoho závislostí a příliš mnoho odpovědností. God třídy vznikají přirozeně s růstem projektů — třída služby začíná s 5 metodami, získá 3 další během sprintu a než si toho někdo všimne, má 40 metod a dotýká se každé části systému.</p>
<p>God třídy jsou jedním z nejsilnějších prediktorů hustoty chyb. Výzkumy konzistentně ukazují, že třídy s vysokým počtem metod a vysokou vazbou mají nepřiměřeně vysokou pravděpodobnost obsahovat defekty.</p>

<h2 id="detection-criteria">Kritéria detekce</h2>
<ul>
<li><strong>Práh počtu metod</strong> — Třídy s 15 nebo více metodami jsou označeny. Třídy nad touto velikostí vykazují výrazně vyšší míru defektů.</li>
<li><strong>Vysoký počet metod + závislostí</strong> — Třídy s velkým počtem metod A velkým počtem závislostí jsou silnějšími kandidáty na god třídu.</li>
<li><strong>Skóre nestability (Ca/Ce)</strong> — AigisCode vypočítává Martinovu metriku nestability: <code>I = Ce / (Ca + Ce)</code>. Třídy s extrémním skóre nestability v kombinaci s vysokým počtem metod indikují architektonické problémy.</li>
</ul>

<h2 id="instability-metric">Metrika nestability</h2>
<ul>
<li><strong>I = 0 (maximálně stabilní)</strong> — Mnoho věcí závisí na této třídě. Pokud je to god třída, změny mají maximální dopad. Refaktorujte tuto jako první.</li>
<li><strong>I = 1 (maximálně nestabilní)</strong> — Třída závisí na mnoha věcech, ale nic na ní nezávisí. Riziková, ale s omezeným dopadem.</li>
<li><strong>I = 0,5 (vyvážená)</strong> — Stejný počet příchozích a odchozích závislostí.</li>
</ul>

<h2 id="refactoring-strategies">Strategie refaktoringu</h2>
<ul>
<li><strong>Extrakce tříd služeb</strong> — Seskupte související metody do specializovaných tříd služeb.</li>
<li><strong>Strategy pattern</strong> — Extrahujte variace stejné operace do strategy objektů.</li>
<li><strong>Facade pattern</strong> — Zachovejte god třídu jako tenkou fasádu delegující na menší třídy.</li>
<li><strong>Dekompozice řízená událostmi</strong> — Nahraďte přímé volání metod událostmi.</li>
</ul>`,
      fr: `<h2 id="what-are-god-classes">Que sont les classes dieu ?</h2>
<p>Une classe dieu est une classe qui fait trop de choses. Elle a trop de methodes, trop de dependances et trop de responsabilites. Les classes dieu emergent naturellement a mesure que les projets grandissent — une classe de service commence avec 5 methodes, en gagne 3 de plus pendant un sprint, et avant que quiconque ne le remarque, elle a 40 methodes.</p>
<p>Les classes dieu sont l'un des predicteurs les plus forts de densite de bugs. Les recherches montrent que les classes avec un nombre eleve de methodes et un couplage eleve sont disproportionnellement susceptibles de contenir des defauts.</p>

<h2 id="detection-criteria">Criteres de detection</h2>
<ul>
<li><strong>Seuil de nombre de methodes</strong> — Les classes avec 15 methodes ou plus sont signalees. Les classes au-dessus de cette taille presentent des taux de defauts significativement plus eleves.</li>
<li><strong>Nombre eleve de methodes + dependances</strong> — Les classes avec un grand nombre de methodes ET de dependances sont des candidates plus fortes.</li>
<li><strong>Scores d'instabilite (Ca/Ce)</strong> — AigisCode calcule la metrique d'instabilite de Martin : <code>I = Ce / (Ca + Ce)</code>. Les classes avec des scores extremes combines a un nombre eleve de methodes indiquent des problemes architecturaux.</li>
</ul>

<h2 id="instability-metric">Metrique d'instabilite</h2>
<ul>
<li><strong>I = 0 (maximalement stable)</strong> — Beaucoup de choses dependent de cette classe. Si c'est une classe dieu, les changements ont un impact maximal. A refactorer en priorite.</li>
<li><strong>I = 1 (maximalement instable)</strong> — Cette classe depend de beaucoup de choses, mais rien ne depend d'elle. Risquee mais impact limite.</li>
<li><strong>I = 0.5 (equilibree)</strong> — Dependances entrantes et sortantes egales.</li>
</ul>

<h2 id="refactoring-strategies">Strategies de refactoring</h2>
<ul>
<li><strong>Extraire des classes de service</strong> — Groupez les methodes liees dans des classes de service specialisees.</li>
<li><strong>Pattern Strategy</strong> — Extrayez les variations d'une meme operation dans des objets strategy.</li>
<li><strong>Pattern Facade</strong> — Gardez la classe dieu comme une facade legere deleguant a des classes plus petites.</li>
<li><strong>Decomposition evenementielle</strong> — Remplacez les appels directs de methodes par des evenements.</li>
</ul>`,
      es: `<h2 id="what-are-god-classes">Que son las clases dios?</h2>
<p>Una clase dios es una clase que hace demasiado. Tiene demasiados metodos, demasiadas dependencias y demasiadas responsabilidades. Las clases dios emergen naturalmente a medida que los proyectos crecen — una clase de servicio comienza con 5 metodos, gana 3 mas durante un sprint, y antes de que alguien lo note, tiene 40 metodos.</p>
<p>Las clases dios son uno de los predictores mas fuertes de densidad de bugs. La investigacion muestra consistentemente que las clases con alto numero de metodos y alto acoplamiento son desproporcionadamente propensas a contener defectos.</p>

<h2 id="detection-criteria">Criterios de deteccion</h2>
<ul>
<li><strong>Umbral de numero de metodos</strong> — Las clases con 15 o mas metodos son marcadas. Las clases por encima de este tamano exhiben tasas de defectos significativamente mas altas.</li>
<li><strong>Alto numero de metodos + dependencias</strong> — Las clases con un gran numero de metodos Y dependencias son candidatas mas fuertes.</li>
<li><strong>Puntuaciones de inestabilidad (Ca/Ce)</strong> — AigisCode calcula la metrica de inestabilidad de Martin: <code>I = Ce / (Ca + Ce)</code>. Las clases con puntuaciones extremas combinadas con alto numero de metodos indican problemas arquitectonicos.</li>
</ul>

<h2 id="instability-metric">Metrica de inestabilidad</h2>
<ul>
<li><strong>I = 0 (maximamente estable)</strong> — Muchas cosas dependen de esta clase. Si es una clase dios, los cambios tienen maximo impacto. Refactorice esto primero.</li>
<li><strong>I = 1 (maximamente inestable)</strong> — Esta clase depende de muchas cosas, pero nada depende de ella. Riesgosa pero con impacto limitado.</li>
<li><strong>I = 0.5 (equilibrada)</strong> — Dependencias entrantes y salientes iguales.</li>
</ul>

<h2 id="refactoring-strategies">Estrategias de refactorizacion</h2>
<ul>
<li><strong>Extraer clases de servicio</strong> — Agrupe metodos relacionados en clases de servicio especializadas.</li>
<li><strong>Patron Strategy</strong> — Extraiga variaciones de la misma operacion en objetos strategy.</li>
<li><strong>Patron Facade</strong> — Mantenga la clase dios como una fachada delgada que delega a clases mas pequenas.</li>
<li><strong>Descomposicion orientada a eventos</strong> — Reemplace las llamadas directas a metodos con eventos.</li>
</ul>`,
      zh: `<h2 id="what-are-god-classes">什么是上帝类？</h2>
<p>上帝类是一个做了太多事情的类。它有太多方法、太多依赖和太多职责。上帝类随着项目增长自然出现——一个服务类从5个方法开始，在一个迭代中增加3个，在任何人注意到之前，它已经有40个方法并触及系统的每个部分。</p>
<p>上帝类是Bug密度最强的预测指标之一。研究一致表明，方法数量多且耦合度高的类更容易包含缺陷。</p>

<h2 id="detection-criteria">检测标准</h2>
<ul>
<li><strong>方法数量阈值</strong> — 15个或更多方法的类被标记。超过此大小的类表现出显著更高的缺陷率。</li>
<li><strong>高方法数+依赖数</strong> — 同时具有大量方法和大量依赖的类是更强的上帝类候选者。</li>
<li><strong>不稳定性评分(Ca/Ce)</strong> — AigisCode计算Martin的不稳定性指标：<code>I = Ce / (Ca + Ce)</code>。极端不稳定性评分与高方法数结合表明架构问题。</li>
</ul>

<h2 id="instability-metric">不稳定性指标</h2>
<ul>
<li><strong>I = 0（最大稳定）</strong> — 很多东西依赖此类。如果它也是上帝类，变更影响最大。优先重构。</li>
<li><strong>I = 1（最大不稳定）</strong> — 此类依赖很多东西，但没有东西依赖它。有风险但影响有限。</li>
<li><strong>I = 0.5（平衡）</strong> — 入站和出站依赖相等。</li>
</ul>

<h2 id="refactoring-strategies">重构策略</h2>
<ul>
<li><strong>提取服务类</strong> — 将相关方法分组到专注的服务类中。</li>
<li><strong>策略模式</strong> — 将同一操作的变体提取到策略对象中。</li>
<li><strong>外观模式</strong> — 保持上帝类作为委托给更小类的薄外观层。</li>
<li><strong>事件驱动分解</strong> — 用事件替换直接方法调用。</li>
</ul>`,
      hi: `<h2 id="what-are-god-classes">गॉड क्लास क्या हैं?</h2>
<p>गॉड क्लास एक ऐसी क्लास है जो बहुत अधिक काम करती है। इसमें बहुत अधिक मेथड, बहुत अधिक डिपेंडेंसी और बहुत अधिक जिम्मेदारियां होती हैं। गॉड क्लास प्रोजेक्ट के बढ़ने के साथ स्वाभाविक रूप से उभरती हैं — एक सर्विस क्लास 5 मेथड से शुरू होती है, एक स्प्रिंट में 3 और जुड़ जाती हैं, और किसी के ध्यान देने से पहले इसमें 40 मेथड हो जाते हैं।</p>
<p>गॉड क्लास बग डेंसिटी के सबसे मजबूत प्रेडिक्टर में से एक हैं। शोध लगातार दिखाता है कि उच्च मेथड काउंट और उच्च कपलिंग वाली क्लास में दोष होने की असमान रूप से अधिक संभावना होती है।</p>

<h2 id="detection-criteria">डिटेक्शन मापदंड</h2>
<ul>
<li><strong>मेथड काउंट थ्रेशोल्ड</strong> — 15 या अधिक मेथड वाली क्लास फ़्लैग की जाती हैं। इस आकार से ऊपर की क्लास काफी अधिक दोष दर दिखाती हैं।</li>
<li><strong>उच्च मेथड + डिपेंडेंसी काउंट</strong> — बड़ी संख्या में मेथड और डिपेंडेंसी दोनों वाली क्लास मजबूत गॉड क्लास कैंडिडेट होती हैं।</li>
<li><strong>इंस्टेबिलिटी स्कोर (Ca/Ce)</strong> — AigisCode Martin की इंस्टेबिलिटी मेट्रिक की गणना करता है: <code>I = Ce / (Ca + Ce)</code>। उच्च मेथड काउंट के साथ एक्सट्रीम स्कोर आर्किटेक्चरल समस्याओं का संकेत देते हैं।</li>
</ul>

<h2 id="instability-metric">इंस्टेबिलिटी मेट्रिक</h2>
<ul>
<li><strong>I = 0 (अधिकतम स्टेबल)</strong> — बहुत कुछ इस क्लास पर निर्भर करता है। यदि यह गॉड क्लास भी है, तो बदलाव का प्रभाव अधिकतम होता है। इसे पहले रिफैक्टर करें।</li>
<li><strong>I = 1 (अधिकतम अनस्टेबल)</strong> — यह क्लास बहुत चीजों पर निर्भर करती है, लेकिन कुछ भी इस पर निर्भर नहीं करता। जोखिमपूर्ण लेकिन सीमित प्रभाव।</li>
<li><strong>I = 0.5 (संतुलित)</strong> — समान इनबाउंड और आउटबाउंड डिपेंडेंसी।</li>
</ul>

<h2 id="refactoring-strategies">रिफैक्टरिंग रणनीतियां</h2>
<ul>
<li><strong>सर्विस क्लास एक्सट्रैक्ट करें</strong> — संबंधित मेथड को विशेष सर्विस क्लास में ग्रुप करें।</li>
<li><strong>स्ट्रैटेजी पैटर्न</strong> — एक ही ऑपरेशन की विविधताओं को स्ट्रैटेजी ऑब्जेक्ट में एक्सट्रैक्ट करें।</li>
<li><strong>फ़ैसेड पैटर्न</strong> — गॉड क्लास को छोटी क्लास को डेलीगेट करने वाले पतले फ़ैसेड के रूप में रखें।</li>
<li><strong>इवेंट-ड्रिवन डीकंपोज़िशन</strong> — डायरेक्ट मेथड कॉल को इवेंट से बदलें।</li>
</ul>`,
      pt: `<h2 id="what-are-god-classes">O que sao god classes?</h2>
<p>Uma god class e uma classe que faz demais. Tem muitos metodos, muitas dependencias e muitas responsabilidades. God classes emergem naturalmente conforme os projetos crescem — uma classe de servico comeca com 5 metodos, ganha mais 3 durante um sprint, e antes que alguem perceba, tem 40 metodos.</p>
<p>God classes sao um dos mais fortes preditores de densidade de bugs. Pesquisas mostram consistentemente que classes com alto numero de metodos e alto acoplamento sao desproporcionalmente propensas a conter defeitos.</p>

<h2 id="detection-criteria">Criterios de deteccao</h2>
<ul>
<li><strong>Limite de numero de metodos</strong> — Classes com 15 ou mais metodos sao sinalizadas. Classes acima deste tamanho exibem taxas de defeitos significativamente mais altas.</li>
<li><strong>Alto numero de metodos + dependencias</strong> — Classes com grande numero de metodos E dependencias sao candidatas mais fortes.</li>
<li><strong>Pontuacoes de instabilidade (Ca/Ce)</strong> — AigisCode calcula a metrica de instabilidade de Martin: <code>I = Ce / (Ca + Ce)</code>. Pontuacoes extremas combinadas com alto numero de metodos indicam problemas arquiteturais.</li>
</ul>

<h2 id="instability-metric">Metrica de instabilidade</h2>
<ul>
<li><strong>I = 0 (maximamente estavel)</strong> — Muitas coisas dependem desta classe. Se e uma god class, mudancas tem impacto maximo. Refatore primeiro.</li>
<li><strong>I = 1 (maximamente instavel)</strong> — Esta classe depende de muitas coisas, mas nada depende dela. Arriscada mas com impacto limitado.</li>
<li><strong>I = 0.5 (equilibrada)</strong> — Dependencias de entrada e saida iguais.</li>
</ul>

<h2 id="refactoring-strategies">Estrategias de refatoracao</h2>
<ul>
<li><strong>Extrair classes de servico</strong> — Agrupe metodos relacionados em classes de servico especializadas.</li>
<li><strong>Padrao Strategy</strong> — Extraia variacoes da mesma operacao em objetos strategy.</li>
<li><strong>Padrao Facade</strong> — Mantenha a god class como uma fachada fina que delega a classes menores.</li>
<li><strong>Decomposicao orientada a eventos</strong> — Substitua chamadas diretas de metodos por eventos.</li>
</ul>`,
      ar: `<h2 id="what-are-god-classes">ما هي الفئات الضخمة؟</h2>
<p>الفئة الضخمة هي فئة تفعل الكثير. لديها طرق كثيرة جداً وتبعيات كثيرة جداً ومسؤولية كبيرة جداً. تظهر الفئات الضخمة بشكل طبيعي مع نمو المشاريع — تبدأ فئة خدمة بـ ٥ طرق وتنمو إلى ٥٠ لأن إضافة طرق جديدة أسهل من إعادة البناء.</p>

<h2 id="detection-criteria">معايير الاكتشاف</h2>
<p>يحدد AigisCode الفئات الضخمة باستخدام إشارات متعددة:</p>
<ul>
<li><strong>عدد الطرق</strong> — الفئات التي تحتوي على أكثر من ١٥ طريقة (قابل للتكوين عبر السياسة).</li>
<li><strong>الاقتران الوارد (Ca)</strong> — عدد الوحدات الأخرى التي تعتمد على هذه الفئة.</li>
<li><strong>الاقتران الصادر (Ce)</strong> — عدد الوحدات التي تعتمد عليها هذه الفئة.</li>
<li><strong>عدم الاستقرار (I = Ce/(Ca+Ce))</strong> — يتراوح بين ٠ (مستقر تماماً) و١ (غير مستقر تماماً).</li>
</ul>

<h2 id="instability-metric">مقياس عدم الاستقرار</h2>
<p>الفئة الضخمة المستقرة (عدم استقرار منخفض) هي أخطر تركيبة: يعتمد عليها العديد من التابعين وعدد طرقها المفرط يعني تغييرات متكررة. الفئة الضخمة غير المستقرة أقل خطراً لأن وحدات أقل تعتمد عليها.</p>

<h2 id="refactoring">إعادة بناء الفئات الضخمة</h2>
<ul>
<li><strong>استخراج الفئة</strong> — جمّع الطرق ذات الصلة في فئة جديدة ذات مسؤولية واحدة.</li>
<li><strong>نمط الاستراتيجية</strong> — عندما تحتوي الفئة الضخمة على تنويعات لنفس العملية استخرجها إلى تطبيقات استراتيجية.</li>
<li><strong>تركيب الفئة</strong> — استبدل الوراثة بالتركيب بحقن التبعيات.</li>
<li><strong>واجهة إعادة البناء</strong> — حدد واجهة ثم استبدل المراجع المباشرة تدريجياً.</li>
</ul>`,
      pl: `<h2 id="god-classes">Wykrywanie god klas</h2>
<p>God klasy to klasy z nadmierna odpowiedzialnoscia. AigisCode identyfikuje je na podstawie liczby metod, powiazania i niestabilnosci.</p>`,
    },
    capabilities: [
      '15+ method threshold detection',
      'Method + dependency count analysis',
      'Instability scores (Ca/Ce metric)',
      'Robert C. Martin instability metric',
      'Coupling analysis per class',
      'Confidence scoring',
      'Multi-language support',
      'Refactoring priority hints via stability',
    ],
    codeExample: `# Analyze for god classes
aigiscode analyze /path/to/project

# View god class findings
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.god_classes'

# Sort by method count
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.god_classes | sort_by(-.method_count)'`,
    relatedSlugs: ['bottlenecks', 'circular-dependencies', 'layer-violations'],
  },
  /* ---- 6. Bottleneck Detection ---- */
  {
    slug: 'bottlenecks',
    icon: 'Crosshair',
    category: 'Architecture',
    title: {
      en: 'Bottleneck Detection',
      cs: 'Detekce úzkých míst',
      fr: 'Détection des goulots d\'étranglement',
      es: 'Detección de cuellos de botella',
      zh: '瓶颈检测',
      hi: 'बॉटलनेक पहचान',
      pt: 'Detecção de gargalos',
      ar: 'اكتشاف الاختناقات',
      pl: 'Wykrywanie waskich gardel',
      bn: 'বটলনেক ডিটেকশন',
    },
    shortDescription: {
      en: 'Identify files with high betweenness centrality and fan-in coupling that create single points of failure. Changes to bottleneck files have the highest blast radius in your architecture.',
      cs: 'Identifikujte soubory s vysokou mezilehlostí a vysokým fan-in propojením, které vytvářejí jediné body selhání. Změny v souborech úzkých míst mají největší dopad na vaši architekturu.',
      fr: 'Identifiez les fichiers avec une centralité d\'intermédiarité élevée et un couplage fan-in qui créent des points de défaillance uniques. Les modifications des fichiers goulots d\'étranglement ont le rayon d\'impact le plus élevé dans votre architecture.',
      es: 'Identifica archivos con alta centralidad de intermediación y acoplamiento fan-in que crean puntos únicos de fallo. Los cambios en archivos cuello de botella tienen el mayor radio de impacto en tu arquitectura.',
      zh: '识别具有高中介中心性和高扇入耦合的文件，这些文件会造成单点故障。对瓶颈文件的更改在架构中具有最大的影响范围。',
      hi: 'उच्च बिटवीननेस सेंट्रैलिटी और फैन-इन कपलिंग वाली फ़ाइलों की पहचान करें जो सिंगल पॉइंट ऑफ़ फ़ेलियर बनाती हैं। बॉटलनेक फ़ाइलों में परिवर्तन आपकी आर्किटेक्चर में सबसे अधिक ब्लास्ट रेडियस रखते हैं।',
      pt: 'Identifique arquivos com alta centralidade de intermediação e acoplamento fan-in que criam pontos únicos de falha. Alterações em arquivos gargalo têm o maior raio de impacto na sua arquitetura.',
      ar: 'حدد الملفات ذات مركزية الوساطة العالية واقتران fan-in التي تخلق نقاط فشل وحيدة. تحمل التغييرات على ملفات الاختناق أعلى نطاق تأثير في بنيتك المعمارية.',
      pl: 'Identyfikuj pliki o wysokiej centralnosci posrednictwa i powiazaniu fan-in, ktore tworza pojedyncze punkty awarii w Twojej architekturze. AigisCode uzywa metryk grafowych petgraph do ujawniania ukrytych hotspotow powiazan.',
      bn: 'উচ্চ বিটুইননেস সেন্ট্রালিটি এবং ফ্যান-ইন কাপলিং সহ ফাইল চিহ্নিত করুন যা একক ব্যর্থতার পয়েন্ট তৈরি করে। বটলনেক ফাইলে পরিবর্তনের আপনার আর্কিটেকচারে সর্বোচ্চ বিস্ফোরণ ব্যাসার্ধ আছে।',
    },
    metaDescription: {
      en: 'Detect bottleneck files in PHP, Python, TypeScript, and JavaScript codebases. AigisCode uses betweenness centrality and fan-in/fan-out analysis to find high-risk coupling hotspots.',
      cs: 'Detekujte soubory úzkých míst v kódových základnách PHP, Python, TypeScript a JavaScript. AigisCode používá analýzu mezilehlosti a fan-in/fan-out k nalezení vysoce rizikových bodů propojení.',
      fr: 'Détectez les fichiers goulots d\'étranglement dans les bases de code PHP, Python, TypeScript et JavaScript. AigisCode utilise la centralité d\'intermédiarité et l\'analyse fan-in/fan-out pour trouver les points chauds de couplage à haut risque.',
      es: 'Detecta archivos cuello de botella en bases de código PHP, Python, TypeScript y JavaScript. AigisCode utiliza centralidad de intermediación y análisis fan-in/fan-out para encontrar puntos críticos de acoplamiento de alto riesgo.',
      zh: '检测 PHP、Python、TypeScript 和 JavaScript 代码库中的瓶颈文件。AigisCode 使用中介中心性和扇入/扇出分析来发现高风险的耦合热点。',
      hi: 'PHP, Python, TypeScript और JavaScript कोडबेस में बॉटलनेक फ़ाइलों का पता लगाएं। AigisCode बिटवीननेस सेंट्रैलिटी और फैन-इन/फैन-आउट विश्लेषण का उपयोग करके उच्च-जोखिम कपलिंग हॉटस्पॉट खोजता है।',
      pt: 'Detecte arquivos gargalo em bases de código PHP, Python, TypeScript e JavaScript. AigisCode usa centralidade de intermediação e análise fan-in/fan-out para encontrar pontos críticos de acoplamento de alto risco.',
      ar: 'اكتشف ملفات الاختناق في قواعد شيفرة PHP وPython وTypeScript وJavaScript. يستخدم AigisCode مركزية الوساطة وتحليل fan-in/fan-out للعثور على نقاط الاقتران عالية المخاطر.',
      pl: 'Wykrywaj pliki bedace waskimi gardlami w bazach kodu PHP, Python, TypeScript i JavaScript. AigisCode uzywa centralnosci posrednictwa i analizy fan-in do znajdowania ukrytych hotspotow powiazan.',
      bn: 'PHP, Python, TypeScript এবং JavaScript কোডবেসে বটলনেক ফাইল শনাক্ত করুন। AigisCode উচ্চ-ঝুঁকি কাপলিং হটস্পট খুঁজতে বিটুইননেস সেন্ট্রালিটি এবং ফ্যান-ইন/ফ্যান-আউট বিশ্লেষণ ব্যবহার করে।',
    },
    content: {
      en: `<h2 id="what-are-bottlenecks">What Are Bottleneck Files?</h2>
<p>A bottleneck file is a file that sits at a critical junction in your dependency graph. Many other files depend on it, and many dependency paths pass through it. If you change a bottleneck file, the blast radius — the number of files that could be affected — is disproportionately large. Bottleneck files are the single points of failure in your architecture.</p>
<p>Bottlenecks are not inherently bad. A well-designed shared utility module might legitimately have many dependents. The problem arises when a bottleneck file is also unstable (frequently changing) or overly complex (too many responsibilities). The combination of high centrality and high churn is a strong predictor of defects.</p>

<h2 id="how-detection-works">How Detection Works</h2>
<p>AigisCode identifies bottleneck files using two complementary graph metrics:</p>
<ul>
<li><strong>Betweenness centrality</strong> — This is the fraction of all shortest paths in the dependency graph that pass through a given file. A file with high betweenness centrality is a bridge between different parts of the codebase. If it fails or changes, it disrupts connections between modules that depend on it as an intermediary. AigisCode uses petgraph's optimized centrality algorithms to compute this for every file in the graph.</li>
<li><strong>Fan-in analysis</strong> — Fan-in counts how many other files directly depend on (import from) a given file. High fan-in means many dependents. A file with fan-in of 50 has 50 files that import from it — changing its interface affects all 50.</li>
</ul>
<p>AigisCode also computes fan-out (how many files a file depends on) and the combined coupling score. Files with both high fan-in and high fan-out are the riskiest: they are tightly connected in both directions.</p>

<h2 id="centrality-explained">Betweenness Centrality Explained</h2>
<p>Consider a project with 200 files. File X has a betweenness centrality of 0.35, meaning 35% of all shortest dependency paths between any two files pass through File X. This makes File X a critical bridge:</p>
<pre><code>Module A ──→ File X ──→ Module B
Module C ──→ File X ──→ Module D
Module E ──→ File X ──→ Module F
...
</code></pre>
<p>If File X introduces a breaking change, the impact propagates to every module that depends on it directly or transitively. High betweenness centrality is a quantitative measure of this architectural risk.</p>

<h2 id="reading-the-report">Reading the Report</h2>
<p>Bottleneck findings include centrality scores and dependency counts:</p>
<pre><code>{
  "graph_analysis": {
    "bottleneck_files": [
      {
        "file": "src/core/database.ts",
        "betweenness_centrality": 0.342,
        "fan_in": 47,
        "fan_out": 8,
        "confidence": "high"
      }
    ]
  }
}</code></pre>
<p>Sort by betweenness centrality to find the most architecturally critical files. High centrality + high fan-in is the combination that demands the most careful change management.</p>

<h2 id="managing-bottlenecks">Managing Bottleneck Files</h2>
<p>You cannot always eliminate bottlenecks, but you can manage the risk they pose:</p>
<ul>
<li><strong>Stabilize the interface</strong> — If many files depend on a bottleneck, make its public API as stable as possible. Use interfaces or abstract types to decouple dependents from implementation details.</li>
<li><strong>Increase test coverage</strong> — Bottleneck files deserve disproportionate test coverage because their blast radius is so large. Any regression in a bottleneck file has maximum downstream impact.</li>
<li><strong>Split when possible</strong> — If a bottleneck has high centrality because it contains too many unrelated exports, split it into focused modules. This distributes the centrality across multiple files.</li>
<li><strong>Code review gates</strong> — Add CODEOWNERS rules or CI checks that require senior review for changes to files above a centrality threshold.</li>
</ul>

<h2 id="bottlenecks-vs-god-classes">Bottlenecks vs. God Classes</h2>
<p>Bottleneck detection and god class detection are complementary. A god class is about internal complexity (too many methods). A bottleneck is about external coupling (too many dependents). A file can be one without the other, or both. When a file is both a god class and a bottleneck, it is the highest priority for refactoring — maximum internal complexity with maximum external blast radius.</p>`,
      cs: `<h2 id="what-are-bottlenecks">Co jsou úzká místa?</h2>
<p>Soubor úzkého místa je soubor, který sedí na kritickém uzlu ve vašem grafu závislostí. Mnoho dalších souborů na něm závisí a mnoho cest závislostí přes něj prochází. Pokud změníte soubor úzkého místa, dopad — počet souborů, které mohou být ovlivněny — je neúměrně velký.</p>
<p>Úzká místa nejsou inherentně špatná. Problém nastává, když je soubor úzkého místa také nestabilní nebo příliš složitý. Kombinace vysoké centrality a častých změn je silným prediktorem defektů.</p>

<h2 id="how-detection-works">Jak detekce funguje</h2>
<ul>
<li><strong>Betweenness centralita</strong> — Podíl všech nejkratších cest v grafu závislostí procházejících daným souborem. AigisCode používá optimalizované algoritmy centrality petgraph.</li>
<li><strong>Fan-in analýza</strong> — Počítá, kolik dalších souborů přímo závisí na daném souboru. Vysoký fan-in znamená mnoho závislých.</li>
</ul>
<p>AigisCode také vypočítává fan-out a kombinované skóre vazby. Soubory s vysokým fan-in i fan-out jsou nejrizikovější.</p>

<h2 id="managing-bottlenecks">Řízení souborů úzkých míst</h2>
<ul>
<li><strong>Stabilizujte rozhraní</strong> — Udělejte veřejné API co nejstabilnější pomocí rozhraní nebo abstraktních typů.</li>
<li><strong>Zvyšte pokrytí testy</strong> — Soubory úzkých míst si zaslouží neúměrné pokrytí testy kvůli velkému dopadu.</li>
<li><strong>Rozdělte pokud možno</strong> — Pokud úzké místo obsahuje příliš mnoho nesouvisejících exportů, rozdělte jej do specializovaných modulů.</li>
<li><strong>Brány pro code review</strong> — Přidejte pravidla CODEOWNERS nebo CI kontroly vyžadující seniorní review pro změny souborů nad prahem centrality.</li>
</ul>

<h2 id="bottlenecks-vs-god-classes">Úzká místa vs. god třídy</h2>
<p>Detekce úzkých míst a god tříd se doplňuje. God třída je o interní složitosti. Úzké místo je o externím propojení. Když je soubor obojím, je nejvyšší prioritou pro refaktoring.</p>`,
      fr: `<h2 id="what-are-bottlenecks">Que sont les fichiers goulots d'etranglement ?</h2>
<p>Un fichier goulot d'etranglement est un fichier situe a une jonction critique dans votre graphe de dependances. De nombreux autres fichiers en dependent et de nombreux chemins de dependances le traversent. Si vous modifiez un fichier goulot, le rayon d'impact est disproportionnellement large.</p>
<p>Les goulots ne sont pas inheremment mauvais. Le probleme survient lorsqu'un fichier goulot est aussi instable ou trop complexe. La combinaison haute centralite + changements frequents est un fort predicteur de defauts.</p>

<h2 id="how-detection-works">Comment fonctionne la detection</h2>
<ul>
<li><strong>Centralite d'intermediation</strong> — La fraction de tous les plus courts chemins dans le graphe passant par un fichier donne. AigisCode utilise les algorithmes optimises de petgraph.</li>
<li><strong>Analyse fan-in</strong> — Compte combien d'autres fichiers dependent directement d'un fichier. Un fan-in eleve signifie beaucoup de dependants.</li>
</ul>
<p>AigisCode calcule egalement le fan-out et le score de couplage combine. Les fichiers avec un fan-in et fan-out eleves sont les plus risques.</p>

<h2 id="managing-bottlenecks">Gerer les fichiers goulots</h2>
<ul>
<li><strong>Stabiliser l'interface</strong> — Rendez l'API publique aussi stable que possible avec des interfaces ou types abstraits.</li>
<li><strong>Augmenter la couverture de test</strong> — Les fichiers goulots meritent une couverture de test disproportionnee vu leur rayon d'impact.</li>
<li><strong>Diviser si possible</strong> — Si un goulot contient trop d'exports non lies, divisez-le en modules specialises.</li>
<li><strong>Portes de revue de code</strong> — Ajoutez des regles CODEOWNERS ou des verifications CI pour les modifications de fichiers au-dessus d'un seuil de centralite.</li>
</ul>

<h2 id="bottlenecks-vs-god-classes">Goulots vs. classes dieu</h2>
<p>La detection des goulots et des classes dieu est complementaire. Une classe dieu concerne la complexite interne. Un goulot concerne le couplage externe. Quand un fichier est les deux, c'est la plus haute priorite de refactoring.</p>`,
      es: `<h2 id="what-are-bottlenecks">Que son los archivos cuello de botella?</h2>
<p>Un archivo cuello de botella es un archivo que se encuentra en una interseccion critica en su grafo de dependencias. Muchos otros archivos dependen de el y muchas rutas de dependencia lo atraviesan. Si cambia un archivo cuello de botella, el radio de impacto es desproporcionadamente grande.</p>
<p>Los cuellos de botella no son inherentemente malos. El problema surge cuando un archivo cuello de botella tambien es inestable o demasiado complejo. La combinacion de alta centralidad y cambios frecuentes es un fuerte predictor de defectos.</p>

<h2 id="how-detection-works">Como funciona la deteccion</h2>
<ul>
<li><strong>Centralidad de intermediacion</strong> — La fraccion de todos los caminos mas cortos en el grafo que pasan por un archivo dado. AigisCode usa algoritmos optimizados de petgraph.</li>
<li><strong>Analisis fan-in</strong> — Cuenta cuantos otros archivos dependen directamente de un archivo. Alto fan-in significa muchos dependientes.</li>
</ul>
<p>AigisCode tambien calcula fan-out y la puntuacion de acoplamiento combinada. Los archivos con alto fan-in y fan-out son los mas riesgosos.</p>

<h2 id="managing-bottlenecks">Gestionar archivos cuello de botella</h2>
<ul>
<li><strong>Estabilizar la interfaz</strong> — Haga la API publica lo mas estable posible con interfaces o tipos abstractos.</li>
<li><strong>Aumentar cobertura de pruebas</strong> — Los archivos cuello de botella merecen cobertura desproporcionada por su gran radio de impacto.</li>
<li><strong>Dividir cuando sea posible</strong> — Si un cuello de botella contiene demasiadas exportaciones no relacionadas, dividalo en modulos especializados.</li>
<li><strong>Puertas de revision de codigo</strong> — Anade reglas CODEOWNERS o verificaciones CI para cambios en archivos por encima de un umbral de centralidad.</li>
</ul>

<h2 id="bottlenecks-vs-god-classes">Cuellos de botella vs. clases dios</h2>
<p>La deteccion de cuellos de botella y clases dios es complementaria. Una clase dios trata sobre complejidad interna. Un cuello de botella trata sobre acoplamiento externo. Cuando un archivo es ambos, es la maxima prioridad de refactorizacion.</p>`,
      zh: `<h2 id="what-are-bottlenecks">什么是瓶颈文件？</h2>
<p>瓶颈文件是位于依赖图关键节点的文件。许多其他文件依赖它，许多依赖路径通过它。如果更改瓶颈文件，影响范围——可能受影响的文件数量——大得不成比例。</p>
<p>瓶颈本身并不一定是坏的。问题出在瓶颈文件同时也不稳定或过于复杂时。高中心性与频繁变更的组合是缺陷的强预测指标。</p>

<h2 id="how-detection-works">检测原理</h2>
<ul>
<li><strong>中介中心性</strong> — 依赖图中所有最短路径通过给定文件的比例。AigisCode使用petgraph的优化中心性算法。</li>
<li><strong>扇入分析</strong> — 计算有多少其他文件直接依赖给定文件。高扇入意味着多个依赖者。</li>
</ul>
<p>AigisCode还计算扇出和组合耦合分数。同时具有高扇入和高扇出的文件风险最大。</p>

<h2 id="managing-bottlenecks">管理瓶颈文件</h2>
<ul>
<li><strong>稳定接口</strong> — 使用接口或抽象类型使公共API尽可能稳定。</li>
<li><strong>增加测试覆盖</strong> — 瓶颈文件因其大影响范围值得不成比例的测试覆盖。</li>
<li><strong>尽可能拆分</strong> — 如果瓶颈包含太多不相关的导出，将其拆分为专注模块。</li>
<li><strong>代码审查门控</strong> — 为超过中心性阈值的文件添加CODEOWNERS规则或CI检查。</li>
</ul>

<h2 id="bottlenecks-vs-god-classes">瓶颈 vs. 上帝类</h2>
<p>瓶颈检测和上帝类检测是互补的。上帝类关注内部复杂性。瓶颈关注外部耦合。当一个文件同时是两者时，它是重构的最高优先级。</p>`,
      hi: `<h2 id="what-are-bottlenecks">बॉटलनेक फ़ाइलें क्या हैं?</h2>
<p>बॉटलनेक फ़ाइल वह फ़ाइल है जो आपके डिपेंडेंसी ग्राफ में एक महत्वपूर्ण जंक्शन पर बैठती है। कई अन्य फ़ाइलें इस पर निर्भर करती हैं और कई डिपेंडेंसी पाथ इसके माध्यम से गुजरते हैं। यदि आप बॉटलनेक फ़ाइल बदलते हैं, तो ब्लास्ट रेडियस असमान रूप से बड़ा होता है।</p>
<p>बॉटलनेक स्वाभाविक रूप से बुरे नहीं होते। समस्या तब उत्पन्न होती है जब बॉटलनेक फ़ाइल अस्थिर या अत्यधिक जटिल भी हो। उच्च सेंट्रैलिटी और बार-बार बदलाव का संयोजन दोषों का मजबूत प्रेडिक्टर है।</p>

<h2 id="how-detection-works">डिटेक्शन कैसे काम करता है</h2>
<ul>
<li><strong>बिटवीननेस सेंट्रैलिटी</strong> — डिपेंडेंसी ग्राफ में किसी दिए गए फ़ाइल से गुजरने वाले सभी सबसे छोटे पाथ का अंश। AigisCode petgraph के ऑप्टिमाइज़्ड एल्गोरिथम का उपयोग करता है।</li>
<li><strong>फैन-इन एनालिसिस</strong> — गिनती करता है कि कितनी अन्य फ़ाइलें सीधे किसी फ़ाइल पर निर्भर करती हैं। उच्च फैन-इन का मतलब है कई डिपेंडेंट।</li>
</ul>
<p>AigisCode फैन-आउट और संयुक्त कपलिंग स्कोर भी गणना करता है। उच्च फैन-इन और फैन-आउट दोनों वाली फ़ाइलें सबसे जोखिमपूर्ण होती हैं।</p>

<h2 id="managing-bottlenecks">बॉटलनेक फ़ाइलों का प्रबंधन</h2>
<ul>
<li><strong>इंटरफ़ेस को स्थिर करें</strong> — इंटरफ़ेस या एब्स्ट्रैक्ट टाइप्स का उपयोग करके पब्लिक API को यथासंभव स्थिर बनाएं।</li>
<li><strong>टेस्ट कवरेज बढ़ाएं</strong> — बॉटलनेक फ़ाइलें अपने बड़े ब्लास्ट रेडियस के कारण असमान टेस्ट कवरेज की हकदार हैं।</li>
<li><strong>जब संभव हो विभाजित करें</strong> — यदि बॉटलनेक में बहुत अधिक असंबंधित एक्सपोर्ट हैं, तो इसे विशेष मॉड्यूल में विभाजित करें।</li>
<li><strong>कोड रिव्यू गेट</strong> — सेंट्रैलिटी थ्रेशोल्ड से ऊपर की फ़ाइलों में बदलाव के लिए CODEOWNERS नियम या CI चेक जोड़ें।</li>
</ul>

<h2 id="bottlenecks-vs-god-classes">बॉटलनेक vs. गॉड क्लास</h2>
<p>बॉटलनेक डिटेक्शन और गॉड क्लास डिटेक्शन पूरक हैं। गॉड क्लास आंतरिक जटिलता के बारे में है। बॉटलनेक बाहरी कपलिंग के बारे में है। जब कोई फ़ाइल दोनों हो, तो यह रिफैक्टरिंग की सर्वोच्च प्राथमिकता है।</p>`,
      pt: `<h2 id="what-are-bottlenecks">O que sao arquivos gargalo?</h2>
<p>Um arquivo gargalo e um arquivo que fica em uma juncao critica no seu grafo de dependencias. Muitos outros arquivos dependem dele e muitos caminhos de dependencia passam por ele. Se voce alterar um arquivo gargalo, o raio de impacto e desproporcionalmente grande.</p>
<p>Gargalos nao sao inerentemente ruins. O problema surge quando um arquivo gargalo tambem e instavel ou excessivamente complexo. A combinacao de alta centralidade e mudancas frequentes e um forte preditor de defeitos.</p>

<h2 id="how-detection-works">Como a deteccao funciona</h2>
<ul>
<li><strong>Centralidade de intermediacao</strong> — A fracao de todos os caminhos mais curtos no grafo que passam por um dado arquivo. AigisCode usa algoritmos otimizados do petgraph.</li>
<li><strong>Analise fan-in</strong> — Conta quantos outros arquivos dependem diretamente de um arquivo. Alto fan-in significa muitos dependentes.</li>
</ul>
<p>AigisCode tambem calcula fan-out e a pontuacao de acoplamento combinada. Arquivos com alto fan-in e fan-out sao os mais arriscados.</p>

<h2 id="managing-bottlenecks">Gerenciar arquivos gargalo</h2>
<ul>
<li><strong>Estabilizar a interface</strong> — Torne a API publica o mais estavel possivel com interfaces ou tipos abstratos.</li>
<li><strong>Aumentar cobertura de testes</strong> — Arquivos gargalo merecem cobertura de testes desproporcional pelo seu grande raio de impacto.</li>
<li><strong>Dividir quando possivel</strong> — Se um gargalo contem muitas exportacoes nao relacionadas, divida-o em modulos especializados.</li>
<li><strong>Portoes de revisao de codigo</strong> — Adicione regras CODEOWNERS ou verificacoes CI para mudancas em arquivos acima de um limite de centralidade.</li>
</ul>

<h2 id="bottlenecks-vs-god-classes">Gargalos vs. god classes</h2>
<p>Deteccao de gargalos e god classes e complementar. Uma god class trata de complexidade interna. Um gargalo trata de acoplamento externo. Quando um arquivo e ambos, e a mais alta prioridade de refatoracao.</p>`,
      ar: `<h2 id="what-are-bottlenecks">ما هي ملفات الاختناق؟</h2>
<p>ملف الاختناق هو ملف يقع عند تقاطع حرج في رسم التبعيات البياني. تعتمد عليه ملفات عديدة وتمر عبره مسارات تبعية كثيرة. عندما يتغير ملف الاختناق يكون نطاق التأثير هائلاً.</p>

<h2 id="detection-metrics">مقاييس الاكتشاف</h2>
<p>يحدد AigisCode ملفات الاختناق باستخدام مقياسين متكاملين:</p>
<ul>
<li><strong>مركزية الوساطة</strong> — نسبة أقصر المسارات بين أي ملفين تمر عبر هذا الملف. مركزية ٠.٣٥ تعني أن ٣٥% من جميع المسارات تمر عبره.</li>
<li><strong>fan-in</strong> — عدد الملفات التي تستورد مباشرة من هذا الملف. fan-in عالي يعني أن التغييرات تؤثر على ملفات عديدة.</li>
</ul>
<p>الملفات ذات fan-in العالي وfan-out العالي هي أخطر الاختناقات لأنها تركّز التبعية وتنشرها في الوقت نفسه.</p>

<h2 id="real-world-example">مثال واقعي</h2>
<p>في مشروع يحتوي على ٢٠٠ ملف: الملف X لديه مركزية وساطة ٠.٣٥ وfan-in بقيمة ٤٠. إذا أدخل تغييراً جذرياً ينتشر التأثير إلى نصف قاعدة الشيفرة. يُنبّه AigisCode إلى هذا الخطر الهيكلي حتى عندما تبدو المقاييس التقليدية طبيعية.</p>

<h2 id="managing-bottlenecks">إدارة ملفات الاختناق</h2>
<ul>
<li><strong>زيادة تغطية الاختبارات</strong> — ملفات الاختناق تحتاج أعلى تغطية اختبار لأن تأثير الانحدار هو الأكبر.</li>
<li><strong>تجميد الواجهة</strong> — حدد واجهات عامة واضحة وقلل التغييرات في التوقيعات.</li>
<li><strong>تقسيم المسؤوليات</strong> — قسّم الملفات الكبيرة إلى وحدات مركّزة ذات واجهات محددة.</li>
<li><strong>مراقبة الاتجاهات</strong> — تتبع درجات المركزية مع مرور الوقت لاكتشاف الاختناقات المتنامية مبكراً.</li>
</ul>`,
      pl: `<h2 id="bottlenecks">Wykrywanie waskich gardel</h2>
<p>Pliki waskich gardel tworza pojedyncze punkty awarii. AigisCode uzywa metryk grafowych petgraph do ich identyfikacji.</p>`,
    },
    capabilities: [
      'Betweenness centrality analysis',
      'Fan-in coupling measurement',
      'Fan-out dependency counting',
      'Combined coupling scores',
      'petgraph optimized algorithms',
      'Blast radius estimation',
      'Centrality-based prioritization',
      'Complementary to god class detection',
    ],
    codeExample: `# Analyze for bottleneck files
aigiscode analyze /path/to/project

# View bottleneck findings
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.bottleneck_files'

# Sort by centrality
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.bottleneck_files | sort_by(-.betweenness_centrality)'`,
    relatedSlugs: ['god-classes', 'circular-dependencies', 'orphan-detection'],
  },
  /* ---- 7. Layer Violations ---- */
  {
    slug: 'layer-violations',
    icon: 'TreeStructure',
    category: 'Architecture',
    title: {
      en: 'Layer Violation Detection',
      cs: 'Detekce porušení vrstev',
      fr: 'Détection des violations de couches',
      es: 'Detección de violaciones de capas',
      zh: '层级违规检测',
      hi: 'लेयर उल्लंघन पहचान',
      pt: 'Detecção de violações de camadas',
      ar: 'اكتشاف انتهاكات الطبقات',
      pl: 'Wykrywanie naruszen warstw',
      bn: 'লেয়ার ভায়োলেশন ডিটেকশন',
    },
    shortDescription: {
      en: 'Detect when code breaks established architectural layer boundaries. Models importing controllers, utilities importing views, infrastructure reaching into domain logic — AigisCode enforces your layer architecture.',
      cs: 'Detekujte, když kód porušuje zavedené hranice architektonických vrstev. Modely importující kontroléry, utility importující pohledy, infrastruktura zasahující do doménové logiky — AigisCode vynucuje vaši vrstvovou architekturu.',
      fr: 'Détectez quand le code enfreint les limites des couches architecturales établies. Des modèles important des contrôleurs, des utilitaires important des vues, l\'infrastructure accédant à la logique métier — AigisCode applique votre architecture en couches.',
      es: 'Detecta cuando el código rompe los límites de capas arquitectónicas establecidos. Modelos importando controladores, utilidades importando vistas, infraestructura accediendo a lógica de dominio — AigisCode aplica tu arquitectura de capas.',
      zh: '检测代码何时打破已建立的架构层级边界。模型导入控制器、工具导入视图、基础设施直接访问领域逻辑——AigisCode 强制执行你的分层架构。',
      hi: 'पता लगाएं कि कोड कब स्थापित आर्किटेक्चरल लेयर सीमाओं को तोड़ता है। मॉडल कंट्रोलर इम्पोर्ट कर रहे हैं, यूटिलिटी व्यू इम्पोर्ट कर रही हैं, इंफ्रास्ट्रक्चर डोमेन लॉजिक तक पहुंच रहा है — AigisCode आपकी लेयर आर्किटेक्चर को लागू करता है।',
      pt: 'Detecte quando o código quebra os limites de camadas arquiteturais estabelecidos. Modelos importando controllers, utilitários importando views, infraestrutura acessando lógica de domínio — AigisCode aplica sua arquitetura de camadas.',
      ar: 'اكتشف عندما تكسر الشيفرة حدود الطبقات المعمارية المحددة. النماذج تستورد المتحكمات والأدوات تستورد العروض والبنية التحتية تصل لمنطق المجال — يفرض AigisCode بنية طبقاتك.',
      pl: 'Wykrywaj, gdy kod lamie ustalone granice warstw architektonicznych. Modele importujace kontrolery, moduly narzedziowe siegajace do logiki biznesowej — AigisCode automatycznie egzekwuje Twoje reguly warstw.',
      bn: 'কোড কখন প্রতিষ্ঠিত আর্কিটেকচারাল লেয়ারের সীমানা ভঙ্গ করে তা শনাক্ত করুন। মডেল কন্ট্রোলার ইমপোর্ট করছে, ইউটিলিটি ভিউ ইমপোর্ট করছে, ইনফ্রাস্ট্রাকচার ডোমেইন লজিকে পৌঁছাচ্ছে — AigisCode আপনার লেয়ার আর্কিটেকচার প্রয়োগ করে।',
    },
    metaDescription: {
      en: 'Detect layer violations in PHP, Python, TypeScript, and JavaScript codebases. AigisCode enforces architectural boundaries with configurable layer maps in policy.json.',
      cs: 'Detekujte porušení vrstev v kódových základnách PHP, Python, TypeScript a JavaScript. AigisCode vynucuje architektonické hranice pomocí konfigurovatelných map vrstev v policy.json.',
      fr: 'Détectez les violations de couches dans les bases de code PHP, Python, TypeScript et JavaScript. AigisCode applique les limites architecturales avec des cartes de couches configurables dans policy.json.',
      es: 'Detecta violaciones de capas en bases de código PHP, Python, TypeScript y JavaScript. AigisCode aplica límites arquitectónicos con mapas de capas configurables en policy.json.',
      zh: '检测 PHP、Python、TypeScript 和 JavaScript 代码库中的层级违规。AigisCode 通过 policy.json 中可配置的层级映射来强制执行架构边界。',
      hi: 'PHP, Python, TypeScript और JavaScript कोडबेस में लेयर उल्लंघनों का पता लगाएं। AigisCode policy.json में कॉन्फ़िगर करने योग्य लेयर मैप के साथ आर्किटेक्चरल सीमाओं को लागू करता है।',
      pt: 'Detecte violações de camadas em bases de código PHP, Python, TypeScript e JavaScript. AigisCode aplica limites arquiteturais com mapas de camadas configuráveis em policy.json.',
      ar: 'اكتشف انتهاكات الطبقات في قواعد شيفرة PHP وPython وTypeScript وJavaScript. يفرض AigisCode الحدود المعمارية بخرائط طبقات قابلة للتكوين في policy.json.',
      pl: 'Wykrywaj naruszenia warstw w bazach kodu PHP, Python, TypeScript i JavaScript. AigisCode egzekwuje granice warstw architektonicznych z konfigurowalnymi definicjami warstw w policy.json.',
      bn: 'PHP, Python, TypeScript এবং JavaScript কোডবেসে লেয়ার ভায়োলেশন শনাক্ত করুন। AigisCode policy.json-এ কনফিগারযোগ্য লেয়ার ম্যাপ সহ আর্কিটেকচারাল সীমানা প্রয়োগ করে।',
    },
    content: {
      en: `<h2 id="what-are-layer-violations">What Are Layer Violations?</h2>
<p>Layered architecture is one of the most fundamental patterns in software design. The principle is simple: higher layers can depend on lower layers, but lower layers must not depend on higher layers. Controllers can import models, but models must not import controllers. Services can use repositories, but repositories must not call services. Views can access view-models, but view-models must not reach into view logic.</p>
<p>When these boundaries are violated, the architecture's properties break down. A model that imports a controller can no longer be tested without instantiating the entire request pipeline. A repository that calls a service creates hidden coupling that makes the data layer unpredictable. Layer violations are the architectural equivalent of water damage — invisible at first, but corrosive over time.</p>

<h2 id="configurable-layers">Configurable Layer Map</h2>
<p>AigisCode does not assume a fixed layer hierarchy. Instead, you define your project's layer structure in <code>policy.json</code> using a <code>layer_map</code>. Each layer has a numeric level, and the rule is simple: a file at level N must not import from a file at level N+1 or higher.</p>
<pre><code>{
  "graph": {
    "layer_map": {
      "src/domain/": 0,
      "src/infrastructure/": 1,
      "src/application/": 2,
      "src/presentation/": 3,
      "src/controllers/": 3,
      "src/routes/": 4
    }
  }
}</code></pre>
<p>With this configuration, domain code (level 0) must not import from infrastructure, application, presentation, or route code. Infrastructure (level 1) can import domain but must not import application or presentation. Routes (level 4) can import from any lower layer.</p>
<p>The layer map uses path prefixes. A file at <code>src/domain/entities/User.ts</code> is assigned to level 0 because its path starts with <code>src/domain/</code>. Files that do not match any layer prefix are not checked for layer violations.</p>

<h2 id="common-violations">Common Violations</h2>
<p>Layer violations follow predictable patterns across different architectures:</p>
<ul>
<li><strong>Model imports controller</strong> — A data model references a controller for convenience (e.g., to access request context). This couples the data layer to the HTTP layer.</li>
<li><strong>Utility imports view</strong> — A shared utility module imports a view component to reuse a rendering function. This prevents the utility from being used in non-UI contexts.</li>
<li><strong>Domain imports infrastructure</strong> — A domain entity directly calls a database query or HTTP client instead of going through a repository interface. This violates clean architecture's dependency rule.</li>
<li><strong>Shared code imports application-specific code</strong> — A library or shared package imports from an application module, reversing the intended dependency direction.</li>
</ul>

<h2 id="detection-process">Detection Process</h2>
<p>Layer violation detection runs during the Graph analysis stage:</p>
<ol>
<li><strong>Layer assignment</strong> — Each file in the dependency graph is assigned a layer level based on path matching against the layer map.</li>
<li><strong>Edge analysis</strong> — Every edge in the dependency graph (import relationship) is checked. If a file at level N imports from a file at level M where M &gt; N, it is a layer violation.</li>
<li><strong>Violation reporting</strong> — Each violation includes the source file, target file, their respective layers, and the import statement that creates the violation.</li>
</ol>

<h2 id="framework-patterns">Framework-Specific Patterns</h2>
<p>Different frameworks have different conventional layer structures. AigisCode's plugin profiles can provide default layer maps:</p>
<ul>
<li><strong>Django</strong> — Models (0) < Managers (1) < Views/Serializers (2) < URLs (3)</li>
<li><strong>Laravel</strong> — Models (0) < Repositories (1) < Services (2) < Controllers (3) < Routes (4)</li>
<li><strong>Express/NestJS</strong> — Entities (0) < Repositories (1) < Services (2) < Controllers (3) < Routes (4)</li>
</ul>
<p>These defaults can be overridden in your project's policy. The layer map is always under your control.</p>

<h2 id="enforcement">Enforcement Strategy</h2>
<p>Layer violations are best caught early. Integrate AigisCode into your CI pipeline to prevent new violations from merging. For existing projects with many violations, use the baseline approach: document current violations and prevent new ones from being introduced while gradually fixing the existing ones.</p>`,
      cs: `<h2 id="what-are-layer-violations">Co jsou porušení vrstev?</h2>
<p>Vrstvená architektura je jedním z nejzákladnějších vzorů v návrhu softwaru. Princip je jednoduchý: vyšší vrstvy mohou záviset na nižších vrstvách, ale nižší vrstvy nesmí záviset na vyšších. Kontroléry mohou importovat modely, ale modely nesmí importovat kontroléry. Když jsou tyto hranice porušeny, vlastnosti architektury se rozpadají.</p>

<h2 id="configurable-layers">Konfigurovatelná mapa vrstev</h2>
<p>AigisCode nepředpokládá pevnou hierarchii vrstev. Místo toho definujete strukturu vrstev vašeho projektu v <code>policy.json</code> pomocí <code>layer_map</code>. Každá vrstva má číselnou úroveň a pravidlo je jednoduché: soubor na úrovni N nesmí importovat ze souboru na úrovni N+1 nebo vyšší.</p>
<pre><code>{
  "graph": {
    "layer_map": {
      "src/domain/": 0,
      "src/infrastructure/": 1,
      "src/application/": 2,
      "src/controllers/": 3
    }
  }
}</code></pre>

<h2 id="common-violations">Běžná porušení</h2>
<ul>
<li><strong>Model importuje kontrolér</strong> — Datový model odkazuje na kontrolér, čímž propojuje datovou vrstvu s HTTP vrstvou.</li>
<li><strong>Utilita importuje pohled</strong> — Sdílený modul utility importuje komponentu pohledu.</li>
<li><strong>Doména importuje infrastrukturu</strong> — Doménová entita přímo volá databázový dotaz místo přes rozhraní repozitáře.</li>
</ul>

<h2 id="framework-patterns">Vzory specifické pro framework</h2>
<p>AigisCode pluginy poskytují výchozí mapy vrstev: Django, Laravel, Express/NestJS. Tyto výchozí hodnoty lze přepsat ve vaší politice projektu.</p>

<h2 id="enforcement">Strategie vynucení</h2>
<p>Porušení vrstev je nejlepší zachytit brzy. Integrujte AigisCode do vašeho CI pipeline. Pro existující projekty s mnoha porušeními použijte baseline přístup: zdokumentujte současná porušení a zabraňte novým.</p>`,
      fr: `<h2 id="what-are-layer-violations">Que sont les violations de couches ?</h2>
<p>L'architecture en couches est l'un des patterns les plus fondamentaux en conception logicielle. Le principe est simple : les couches superieures peuvent dependre des couches inferieures, mais pas l'inverse. Les controleurs peuvent importer des modeles, mais les modeles ne doivent pas importer des controleurs. Quand ces limites sont violees, les proprietes de l'architecture se degradent.</p>

<h2 id="configurable-layers">Carte de couches configurable</h2>
<p>AigisCode ne suppose pas une hierarchie fixe. Vous definissez la structure de couches dans <code>policy.json</code> via <code>layer_map</code>. Chaque couche a un niveau numerique et la regle est simple : un fichier au niveau N ne doit pas importer depuis un fichier au niveau N+1 ou superieur.</p>
<pre><code>{
  "graph": {
    "layer_map": {
      "src/domain/": 0,
      "src/infrastructure/": 1,
      "src/application/": 2,
      "src/controllers/": 3
    }
  }
}</code></pre>

<h2 id="common-violations">Violations courantes</h2>
<ul>
<li><strong>Modele importe un controleur</strong> — Un modele de donnees reference un controleur, couplant la couche donnees a la couche HTTP.</li>
<li><strong>Utilitaire importe une vue</strong> — Un module utilitaire partage importe un composant de vue.</li>
<li><strong>Domaine importe l'infrastructure</strong> — Une entite de domaine appelle directement une requete base de donnees au lieu de passer par une interface de repository.</li>
</ul>

<h2 id="framework-patterns">Patterns specifiques aux frameworks</h2>
<p>Les plugins AigisCode fournissent des cartes de couches par defaut : Django, Laravel, Express/NestJS. Ces defauts peuvent etre remplaces dans votre politique de projet.</p>

<h2 id="enforcement">Strategie d'application</h2>
<p>Les violations de couches sont mieux detectees tot. Integrez AigisCode dans votre pipeline CI. Pour les projets existants avec de nombreuses violations, utilisez l'approche de reference : documentez les violations actuelles et empêchez les nouvelles.</p>`,
      es: `<h2 id="what-are-layer-violations">Que son las violaciones de capas?</h2>
<p>La arquitectura en capas es uno de los patrones mas fundamentales en el diseno de software. El principio es simple: las capas superiores pueden depender de las inferiores, pero las inferiores no deben depender de las superiores. Los controladores pueden importar modelos, pero los modelos no deben importar controladores. Cuando se violan estos limites, las propiedades de la arquitectura se degradan.</p>

<h2 id="configurable-layers">Mapa de capas configurable</h2>
<p>AigisCode no asume una jerarquia fija. Usted define la estructura de capas en <code>policy.json</code> via <code>layer_map</code>. Cada capa tiene un nivel numerico y la regla es simple: un archivo en el nivel N no debe importar desde un archivo en el nivel N+1 o superior.</p>
<pre><code>{
  "graph": {
    "layer_map": {
      "src/domain/": 0,
      "src/infrastructure/": 1,
      "src/application/": 2,
      "src/controllers/": 3
    }
  }
}</code></pre>

<h2 id="common-violations">Violaciones comunes</h2>
<ul>
<li><strong>Modelo importa controlador</strong> — Un modelo de datos referencia un controlador, acoplando la capa de datos con la capa HTTP.</li>
<li><strong>Utilidad importa vista</strong> — Un modulo utilitario compartido importa un componente de vista.</li>
<li><strong>Dominio importa infraestructura</strong> — Una entidad de dominio llama directamente a una consulta de base de datos en lugar de usar una interfaz de repositorio.</li>
</ul>

<h2 id="framework-patterns">Patrones especificos de framework</h2>
<p>Los plugins de AigisCode proporcionan mapas de capas por defecto: Django, Laravel, Express/NestJS. Estos valores por defecto se pueden sobrescribir en la politica del proyecto.</p>

<h2 id="enforcement">Estrategia de aplicacion</h2>
<p>Las violaciones de capas se detectan mejor temprano. Integre AigisCode en su pipeline CI. Para proyectos existentes con muchas violaciones, use el enfoque de linea base: documente las violaciones actuales y prevenga nuevas.</p>`,
      zh: `<h2 id="what-are-layer-violations">什么是层级违规？</h2>
<p>分层架构是软件设计中最基本的模式之一。原则很简单：高层可以依赖低层，但低层不能依赖高层。控制器可以导入模型，但模型不能导入控制器。当这些边界被违反时，架构的属性就会崩溃。</p>

<h2 id="configurable-layers">可配置层级映射</h2>
<p>AigisCode不假设固定的层级结构。您在 <code>policy.json</code> 中通过 <code>layer_map</code> 定义项目的层级结构。每层有一个数字级别，规则很简单：N级文件不能从N+1或更高级别的文件导入。</p>
<pre><code>{
  "graph": {
    "layer_map": {
      "src/domain/": 0,
      "src/infrastructure/": 1,
      "src/application/": 2,
      "src/controllers/": 3
    }
  }
}</code></pre>

<h2 id="common-violations">常见违规</h2>
<ul>
<li><strong>模型导入控制器</strong> — 数据模型引用控制器，将数据层与HTTP层耦合。</li>
<li><strong>工具导入视图</strong> — 共享工具模块导入视图组件。</li>
<li><strong>领域导入基础设施</strong> — 领域实体直接调用数据库查询，而不是通过仓库接口。</li>
</ul>

<h2 id="framework-patterns">框架特定模式</h2>
<p>AigisCode插件提供默认层级映射：Django、Laravel、Express/NestJS。这些默认值可以在项目策略中覆盖。</p>

<h2 id="enforcement">执行策略</h2>
<p>层级违规最好尽早捕获。将AigisCode集成到CI流水线中。对于存在大量违规的现有项目，使用基线方法：记录当前违规并防止新违规的引入。</p>`,
      hi: `<h2 id="what-are-layer-violations">लेयर उल्लंघन क्या हैं?</h2>
<p>लेयर्ड आर्किटेक्चर सॉफ्टवेयर डिज़ाइन में सबसे मूलभूत पैटर्न में से एक है। सिद्धांत सरल है: उच्च लेयर निचली लेयर पर निर्भर कर सकती हैं, लेकिन निचली लेयर उच्च लेयर पर निर्भर नहीं होनी चाहिए। कंट्रोलर मॉडल इम्पोर्ट कर सकते हैं, लेकिन मॉडल कंट्रोलर इम्पोर्ट नहीं कर सकते। जब ये सीमाएं टूटती हैं, तो आर्किटेक्चर के गुण खराब हो जाते हैं।</p>

<h2 id="configurable-layers">कॉन्फ़िगर करने योग्य लेयर मैप</h2>
<p>AigisCode एक निश्चित लेयर पदानुक्रम नहीं मानता। आप <code>policy.json</code> में <code>layer_map</code> के माध्यम से अपने प्रोजेक्ट की लेयर संरचना परिभाषित करते हैं। प्रत्येक लेयर का एक संख्यात्मक स्तर होता है और नियम सरल है: स्तर N पर एक फ़ाइल स्तर N+1 या उच्चतर की फ़ाइल से इम्पोर्ट नहीं कर सकती।</p>
<pre><code>{
  "graph": {
    "layer_map": {
      "src/domain/": 0,
      "src/infrastructure/": 1,
      "src/application/": 2,
      "src/controllers/": 3
    }
  }
}</code></pre>

<h2 id="common-violations">सामान्य उल्लंघन</h2>
<ul>
<li><strong>मॉडल कंट्रोलर इम्पोर्ट करता है</strong> — डेटा मॉडल कंट्रोलर को रेफ़रेंस करता है, डेटा लेयर को HTTP लेयर से जोड़ता है।</li>
<li><strong>यूटिलिटी व्यू इम्पोर्ट करती है</strong> — शेयर्ड यूटिलिटी मॉड्यूल व्यू कंपोनेंट इम्पोर्ट करता है।</li>
<li><strong>डोमेन इंफ्रास्ट्रक्चर इम्पोर्ट करता है</strong> — डोमेन एंटिटी रिपॉज़िटरी इंटरफ़ेस के बजाय सीधे डेटाबेस क्वेरी कॉल करती है।</li>
</ul>

<h2 id="framework-patterns">फ्रेमवर्क-विशिष्ट पैटर्न</h2>
<p>AigisCode प्लगइन्स डिफ़ॉल्ट लेयर मैप प्रदान करते हैं: Django, Laravel, Express/NestJS। इन डिफ़ॉल्ट को आपकी प्रोजेक्ट पॉलिसी में ओवरराइड किया जा सकता है।</p>

<h2 id="enforcement">एनफ़ोर्समेंट रणनीति</h2>
<p>लेयर उल्लंघन जल्दी पकड़ना सबसे अच्छा है। AigisCode को अपनी CI पाइपलाइन में इंटीग्रेट करें। कई उल्लंघनों वाले मौजूदा प्रोजेक्ट के लिए, बेसलाइन अप्रोच का उपयोग करें: वर्तमान उल्लंघनों को डॉक्यूमेंट करें और नए उल्लंघनों को रोकें।</p>`,
      pt: `<h2 id="what-are-layer-violations">O que sao violacoes de camadas?</h2>
<p>Arquitetura em camadas e um dos padroes mais fundamentais no design de software. O principio e simples: camadas superiores podem depender de camadas inferiores, mas camadas inferiores nao devem depender de superiores. Controllers podem importar modelos, mas modelos nao devem importar controllers. Quando esses limites sao violados, as propriedades da arquitetura se degradam.</p>

<h2 id="configurable-layers">Mapa de camadas configuravel</h2>
<p>AigisCode nao assume uma hierarquia fixa. Voce define a estrutura de camadas no <code>policy.json</code> via <code>layer_map</code>. Cada camada tem um nivel numerico e a regra e simples: um arquivo no nivel N nao deve importar de um arquivo no nivel N+1 ou superior.</p>
<pre><code>{
  "graph": {
    "layer_map": {
      "src/domain/": 0,
      "src/infrastructure/": 1,
      "src/application/": 2,
      "src/controllers/": 3
    }
  }
}</code></pre>

<h2 id="common-violations">Violacoes comuns</h2>
<ul>
<li><strong>Modelo importa controller</strong> — Um modelo de dados referencia um controller, acoplando a camada de dados a camada HTTP.</li>
<li><strong>Utilitario importa view</strong> — Um modulo utilitario compartilhado importa um componente de view.</li>
<li><strong>Dominio importa infraestrutura</strong> — Uma entidade de dominio chama diretamente uma query de banco ao inves de usar uma interface de repositorio.</li>
</ul>

<h2 id="framework-patterns">Padroes especificos de framework</h2>
<p>Os plugins AigisCode fornecem mapas de camadas padrao: Django, Laravel, Express/NestJS. Esses padroes podem ser substituidos na politica do projeto.</p>

<h2 id="enforcement">Estrategia de aplicacao</h2>
<p>Violacoes de camadas sao melhor detectadas cedo. Integre AigisCode no seu pipeline CI. Para projetos existentes com muitas violacoes, use a abordagem de baseline: documente violacoes atuais e previna novas.</p>`,
      ar: `<h2 id="what-are-layer-violations">ما هي انتهاكات الطبقات؟</h2>
<p>البنية الطبقية من أكثر الأنماط أساسية في تصميم البرمجيات. المبدأ بسيط: الطبقات العليا تعتمد على الطبقات الدنيا لكن الطبقات الدنيا لا يجب أن تعتمد على الطبقات العليا. عندما تُنتهك هذه الحدود تنهار خصائص البنية المعمارية.</p>

<h2 id="configurable-layer-map">خريطة طبقات قابلة للتكوين</h2>
<p>لا يفترض AigisCode تسلسلاً طبقياً ثابتاً. بدلاً من ذلك تحدد هيكل طبقات مشروعك في <code>policy.json</code>:</p>
<ul>
<li><strong>domain</strong> (المستوى ٠) — منطق العمل الأساسي والكيانات.</li>
<li><strong>infrastructure</strong> (المستوى ١) — تطبيقات قاعدة البيانات والخدمات الخارجية.</li>
<li><strong>application</strong> (المستوى ٢) — حالات الاستخدام ومنسقو الخدمات.</li>
<li><strong>presentation/api</strong> (المستوى ٣) — المتحكمات والمسارات وطبقة واجهة المستخدم.</li>
</ul>
<p>أي استيراد من طبقة ذات رقم أعلى إلى طبقة ذات رقم أدنى صالح. الاستيراد في الاتجاه المعاكس انتهاك.</p>

<h2 id="common-violations">الانتهاكات الشائعة</h2>
<ul>
<li><strong>نموذج يستورد متحكماً</strong> — يربط منطق المجال بالعرض.</li>
<li><strong>مستودع يستورد عرضاً</strong> — يخلط طبقة البيانات مع واجهة المستخدم.</li>
<li><strong>أداة تصل لطبقة التطبيق</strong> — تكسر إمكانية إعادة استخدام الأدوات المشتركة.</li>
</ul>

<h2 id="ci-integration">تكامل CI</h2>
<p>من الأفضل اكتشاف انتهاكات الطبقات مبكراً. ادمج AigisCode في خط أنابيب CI لمنع دمج الانتهاكات الجديدة. استخدم خريطة الطبقات لتدوين بنيتك المعمارية المقصودة ثم دع التحليل الآلي يفرضها في كل طلب سحب.</p>`,
      pl: `<h2 id="layer-violations">Naruszenia warstw</h2>
<p>AigisCode egzekwuje granice warstw architektonicznych zdefiniowane w <code>policy.json</code>.</p>`,
    },
    capabilities: [
      'Configurable layer_map in policy.json',
      'Path-prefix-based layer assignment',
      'Directional dependency enforcement',
      'Import-level violation reporting',
      'Framework-specific default layer maps',
      'CI/CD integration for prevention',
      'Baseline approach for existing codebases',
      'Complementary to bottleneck detection',
    ],
    codeExample: `# Define layer boundaries in policy
cat > .aigiscode/policy.json << EOF
{
  "graph": {
    "layer_map": {
      "src/domain/": 0,
      "src/infrastructure/": 1,
      "src/application/": 2,
      "src/controllers/": 3
    }
  }
}
EOF

# Run analysis
aigiscode analyze /path/to/project

# View layer violations
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.layer_violations'`,
    relatedSlugs: ['circular-dependencies', 'god-classes', 'bottlenecks'],
  },
  /* ---- 8. Orphan Detection ---- */
  {
    slug: 'orphan-detection',
    icon: 'FileMagnifyingGlass',
    category: 'Code Quality',
    title: {
      en: 'Orphan File Detection',
      cs: 'Detekce osiřelých souborů',
      fr: 'Détection des fichiers orphelins',
      es: 'Detección de archivos huérfanos',
      zh: '孤立文件检测',
      hi: 'ऑर्फ़न फ़ाइल पहचान',
      pt: 'Detecção de arquivos órfãos',
      ar: 'اكتشاف الملفات اليتيمة',
      pl: 'Wykrywanie osieroconych plikow',
      bn: 'অর্ফান ফাইল ডিটেকশন',
    },
    shortDescription: {
      en: 'Discover files with zero inbound dependencies that may be dead weight in your codebase. AigisCode distinguishes real orphans from legitimate entry points like controllers, commands, and bootstrap files.',
      cs: 'Objevte soubory s nulovými příchozími závislostmi, které mohou být mrtvou zátěží ve vaší kódové základně. AigisCode rozlišuje skutečné sirotky od legitimních vstupních bodů jako kontroléry, příkazy a bootstrap soubory.',
      fr: 'Découvrez les fichiers sans dépendances entrantes qui peuvent être du poids mort dans votre base de code. AigisCode distingue les vrais orphelins des points d\'entrée légitimes comme les contrôleurs, les commandes et les fichiers de démarrage.',
      es: 'Descubre archivos sin dependencias entrantes que pueden ser peso muerto en tu código. AigisCode distingue los verdaderos huérfanos de puntos de entrada legítimos como controladores, comandos y archivos de arranque.',
      zh: '发现零入站依赖的文件，这些文件可能是代码库中的无用负担。AigisCode 区分真正的孤立文件和合法的入口点，如控制器、命令和引导文件。',
      hi: 'शून्य इनबाउंड डिपेंडेंसी वाली फ़ाइलों की खोज करें जो आपके कोडबेस में डेड वेट हो सकती हैं। AigisCode वास्तविक ऑर्फ़न को कंट्रोलर, कमांड और बूटस्ट्रैप फ़ाइलों जैसे वैध एंट्री पॉइंट से अलग करता है।',
      pt: 'Descubra arquivos sem dependências de entrada que podem ser peso morto na sua base de código. AigisCode distingue órfãos reais de pontos de entrada legítimos como controllers, comandos e arquivos de bootstrap.',
      ar: 'اكتشف الملفات بدون تبعيات واردة التي قد تكون عبئاً ميتاً في قاعدة شيفرتك. يميّز AigisCode الملفات اليتيمة الحقيقية عن نقاط الدخول المشروعة مثل المتحكمات والأوامر وملفات التمهيد.',
      pl: 'Odkryj pliki bez przychodzacych zaleznosci, ktore moga byc martwym obciazeniem w Twojej bazie kodu. AigisCode odroznia prawdziwe sieroty od prawowitych punktow wejscia za pomoca konfigurowalnych wzorow wejscia.',
      bn: 'আপনার কোডবেসে ডেড ওয়েট হতে পারে এমন শূন্য ইনবাউন্ড ডিপেন্ডেন্সি সহ ফাইল আবিষ্কার করুন। AigisCode কন্ট্রোলার, কমান্ড এবং বুটস্ট্র্যাপ ফাইলের মতো বৈধ এন্ট্রি পয়েন্ট থেকে প্রকৃত অর্ফান আলাদা করে।',
    },
    metaDescription: {
      en: 'Detect orphan files with zero inbound dependencies in PHP, Python, TypeScript, and JavaScript codebases. AigisCode distinguishes real orphans from entry points using policy-driven patterns.',
      cs: 'Detekujte osiřelé soubory s nulovými příchozími závislostmi v kódových základnách PHP, Python, TypeScript a JavaScript. AigisCode rozlišuje skutečné sirotky od vstupních bodů pomocí vzorů řízených politikou.',
      fr: 'Détectez les fichiers orphelins sans dépendances entrantes dans les bases de code PHP, Python, TypeScript et JavaScript. AigisCode distingue les vrais orphelins des points d\'entrée grâce à des motifs pilotés par les politiques.',
      es: 'Detecta archivos huérfanos sin dependencias entrantes en bases de código PHP, Python, TypeScript y JavaScript. AigisCode distingue los huérfanos reales de los puntos de entrada usando patrones basados en políticas.',
      zh: '检测 PHP、Python、TypeScript 和 JavaScript 代码库中零入站依赖的孤立文件。AigisCode 使用策略驱动的模式区分真正的孤立文件和入口点。',
      hi: 'PHP, Python, TypeScript और JavaScript कोडबेस में शून्य इनबाउंड डिपेंडेंसी वाली ऑर्फ़न फ़ाइलों का पता लगाएं। AigisCode पॉलिसी-ड्रिवन पैटर्न का उपयोग करके वास्तविक ऑर्फ़न को एंट्री पॉइंट से अलग करता है।',
      pt: 'Detecte arquivos órfãos sem dependências de entrada em bases de código PHP, Python, TypeScript e JavaScript. AigisCode distingue órfãos reais de pontos de entrada usando padrões orientados por políticas.',
      ar: 'اكتشف الملفات اليتيمة بدون تبعيات واردة في قواعد شيفرة PHP وPython وTypeScript وJavaScript. يميّز AigisCode الملفات اليتيمة الحقيقية عن نقاط الدخول باستخدام أنماط مدفوعة بالسياسات.',
      pl: 'Wykrywaj osierocone pliki bez przychodzacych zaleznosci w bazach kodu PHP, Python, TypeScript i JavaScript. AigisCode znajduje pliki bedace martwym obciazeniem, do ktorych nie odwoluje sie zaden inny modul.',
      bn: 'PHP, Python, TypeScript এবং JavaScript কোডবেসে শূন্য ইনবাউন্ড ডিপেন্ডেন্সি সহ অর্ফান ফাইল শনাক্ত করুন। AigisCode পলিসি-চালিত প্যাটার্ন ব্যবহার করে এন্ট্রি পয়েন্ট থেকে প্রকৃত অর্ফান আলাদা করে।',
    },
    content: {
      en: `<h2 id="what-are-orphan-files">What Are Orphan Files?</h2>
<p>An orphan file is a file that no other file in your project imports from. It has zero inbound dependencies in the dependency graph — nothing references it, nothing uses its exports, nothing includes it. In a well-organized codebase, most files are connected to the rest of the system through import chains. Files that sit outside this web of connections are either dead code that can be deleted, or entry points that are invoked by something outside the dependency graph (like a CLI runner, web framework, or test harness).</p>
<p>The distinction between orphan-as-dead-code and orphan-as-entry-point is critical. Deleting a dead orphan is a net win. Deleting an entry point breaks the application. AigisCode uses policy-driven entry point patterns to make this distinction automatically.</p>

<h2 id="how-detection-works">How Detection Works</h2>
<p>Orphan detection is part of AigisCode's dependency graph analysis:</p>
<ol>
<li><strong>Build the graph</strong> — The graph builder constructs a directed graph of all files and their import relationships using petgraph.</li>
<li><strong>Compute in-degree</strong> — For each file, count the number of incoming edges (files that import from it). Files with in-degree 0 are orphan candidates.</li>
<li><strong>Apply entry point patterns</strong> — Check each orphan candidate against the configured entry point patterns from <code>policy.json</code>. Files matching entry point patterns are excluded from the orphan list.</li>
<li><strong>Report remaining orphans</strong> — Files with zero inbound dependencies that do not match any entry point pattern are reported as orphans.</li>
</ol>

<h2 id="entry-point-patterns">Entry Point Patterns</h2>
<p>Entry points are files that are invoked externally rather than through the import system. Common entry points include:</p>
<ul>
<li><strong>Controllers</strong> — Web framework controllers are registered with the router, not imported by other files.</li>
<li><strong>CLI commands</strong> — Command classes are discovered by the CLI framework at runtime.</li>
<li><strong>Migration files</strong> — Database migrations are executed by the migration runner, not imported.</li>
<li><strong>Bootstrap files</strong> — Application bootstrap files (like <code>main.ts</code>, <code>app.py</code>, <code>index.php</code>) are the root of the import tree.</li>
<li><strong>Configuration files</strong> — Config files are loaded by the framework, not imported by application code.</li>
<li><strong>Test files</strong> — Test files are run by the test runner, not imported by application code.</li>
</ul>
<p>Configure entry point patterns in your policy to prevent false positives:</p>
<pre><code>{
  "graph": {
    "orphan_entry_patterns": [
      "src/controllers/**/*.ts",
      "src/commands/**/*.ts",
      "src/bootstrap/**/*.ts",
      "database/migrations/**/*.php",
      "tests/**/*"
    ]
  }
}</code></pre>

<h2 id="real-orphans">Identifying Real Orphans</h2>
<p>Once entry points are excluded, the remaining orphans fall into common categories:</p>
<ul>
<li><strong>Deprecated feature files</strong> — Files from features that were removed or replaced, but the files were never deleted. These are the most common real orphans.</li>
<li><strong>Incomplete implementations</strong> — Files that were started but never connected to the rest of the system. They were abandoned mid-development.</li>
<li><strong>Copied utilities</strong> — Utility files copied from another project that were never actually imported. They sit in the directory taking up space and confusing searches.</li>
<li><strong>Generated stubs</strong> — Auto-generated files (like code scaffolding output) that were created but never used.</li>
</ul>

<h2 id="impact">Impact of Removing Orphans</h2>
<p>Removing orphan files has several benefits:</p>
<ul>
<li><strong>Reduced cognitive load</strong> — Fewer files in the project means faster searching, easier navigation, and less confusion for new developers.</li>
<li><strong>Smaller bundle/package size</strong> — In TypeScript/JavaScript projects, orphan files may still be included in bundles if they are in a directory that is glob-imported.</li>
<li><strong>Cleaner CI</strong> — Fewer files to lint, test, and process in CI pipelines.</li>
<li><strong>Reduced security surface</strong> — Unmaintained code is a potential attack vector. Removing it shrinks the security surface.</li>
</ul>

<h2 id="orphans-in-monorepos">Orphans in Monorepos</h2>
<p>In monorepos, orphan detection is especially valuable. Packages that no other package imports from may be completely unused workspace members. AigisCode's cross-package graph analysis identifies these orphan packages, which can then be archived or removed to reduce workspace complexity and build times.</p>`,
      cs: `<h2 id="what-are-orphan-files">Co jsou osiřelé soubory?</h2>
<p>Osiřelý soubor je soubor, který žádný jiný soubor ve vašem projektu neimportuje. Má nulový počet příchozích závislostí v grafu závislostí. Soubory mimo tuto síť propojení jsou buď mrtvý kód, který lze smazat, nebo vstupní body vyvolávané něčím mimo graf závislostí (jako CLI runner, webový framework nebo testovací framework).</p>

<h2 id="how-detection-works">Jak detekce funguje</h2>
<ol>
<li><strong>Sestavení grafu</strong> — Pomocí petgraph se vytvoří orientovaný graf všech souborů a jejich importních vztahů.</li>
<li><strong>Výpočet in-degree</strong> — Pro každý soubor se spočítá počet příchozích hran. Soubory s in-degree 0 jsou kandidáti na orphany.</li>
<li><strong>Aplikace vzorů vstupních bodů</strong> — Každý kandidát se porovná s nakonfigurovanými vzory vstupních bodů z <code>policy.json</code>.</li>
<li><strong>Report zbývajících orphanů</strong> — Soubory, které neodpovídají žádnému vzoru vstupního bodu, jsou hlášeny jako orphany.</li>
</ol>

<h2 id="entry-point-patterns">Vzory vstupních bodů</h2>
<p>Vstupní body jsou soubory vyvolávané externě: kontroléry, CLI příkazy, migrace, bootstrap soubory, konfigurační soubory a testovací soubory. Konfigurujte vzory vstupních bodů v politice:</p>
<pre><code>{
  "graph": {
    "orphan_entry_patterns": [
      "src/controllers/**/*.ts",
      "src/commands/**/*.ts",
      "tests/**/*"
    ]
  }
}</code></pre>

<h2 id="real-orphans">Identifikace skutečných orphanů</h2>
<p>Po vyloučení vstupních bodů zbývající orphany zahrnují: soubory zastaralých funkcí, nedokončené implementace, zkopírované utility a generované stubby.</p>

<h2 id="orphans-in-monorepos">Orphany v monorepech</h2>
<p>V monorepech je detekce orphanů obzvláště cenná. Balíčky, které žádný jiný balíček neimportuje, mohou být zcela nepoužívaní členové workspace.</p>`,
      fr: `<h2 id="what-are-orphan-files">Que sont les fichiers orphelins ?</h2>
<p>Un fichier orphelin est un fichier qu'aucun autre fichier du projet n'importe. Il a zero dependances entrantes dans le graphe. Les fichiers en dehors de ce reseau de connexions sont soit du code mort pouvant etre supprime, soit des points d'entree invoques par quelque chose en dehors du graphe (comme un runner CLI, un framework web ou un harnais de test).</p>

<h2 id="how-detection-works">Comment fonctionne la detection</h2>
<ol>
<li><strong>Construction du graphe</strong> — Un graphe dirige de tous les fichiers et leurs relations d'import est construit avec petgraph.</li>
<li><strong>Calcul du degre entrant</strong> — Pour chaque fichier, compter les aretes entrantes. Les fichiers avec un degre entrant de 0 sont candidats orphelins.</li>
<li><strong>Application des patterns d'entree</strong> — Chaque candidat est verifie contre les patterns configures dans <code>policy.json</code>.</li>
<li><strong>Report des orphelins restants</strong> — Les fichiers ne correspondant a aucun pattern sont reportes comme orphelins.</li>
</ol>

<h2 id="entry-point-patterns">Patterns de points d'entree</h2>
<p>Les points d'entree sont des fichiers invoques en externe : controleurs, commandes CLI, migrations, fichiers d'amorcage, fichiers de configuration et fichiers de test. Configurez les patterns :</p>
<pre><code>{
  "graph": {
    "orphan_entry_patterns": [
      "src/controllers/**/*.ts",
      "src/commands/**/*.ts",
      "tests/**/*"
    ]
  }
}</code></pre>

<h2 id="real-orphans">Identifier les vrais orphelins</h2>
<p>Apres exclusion des points d'entree, les orphelins restants incluent : fichiers de fonctionnalites obsoletes, implementations inachevees, utilitaires copies et stubs generes.</p>

<h2 id="orphans-in-monorepos">Orphelins dans les monorepos</h2>
<p>Dans les monorepos, la detection d'orphelins est particulierement precieuse. Les packages qu'aucun autre package n'importe peuvent etre des membres de workspace completement inutilises.</p>`,
      es: `<h2 id="what-are-orphan-files">Que son los archivos huerfanos?</h2>
<p>Un archivo huerfano es un archivo que ningun otro archivo del proyecto importa. Tiene cero dependencias entrantes en el grafo. Los archivos fuera de esta red de conexiones son codigo muerto que se puede eliminar, o puntos de entrada invocados por algo fuera del grafo (como un runner CLI, un framework web o un arnes de pruebas).</p>

<h2 id="how-detection-works">Como funciona la deteccion</h2>
<ol>
<li><strong>Construir el grafo</strong> — Se construye un grafo dirigido de todos los archivos y sus relaciones de importacion con petgraph.</li>
<li><strong>Calcular grado de entrada</strong> — Para cada archivo, contar aristas entrantes. Los archivos con grado 0 son candidatos huerfanos.</li>
<li><strong>Aplicar patrones de entrada</strong> — Cada candidato se verifica contra los patrones configurados en <code>policy.json</code>.</li>
<li><strong>Reportar huerfanos restantes</strong> — Los archivos que no coinciden con ningun patron se reportan como huerfanos.</li>
</ol>

<h2 id="entry-point-patterns">Patrones de puntos de entrada</h2>
<p>Los puntos de entrada son archivos invocados externamente: controladores, comandos CLI, migraciones, archivos de arranque, archivos de configuracion y archivos de prueba. Configure los patrones:</p>
<pre><code>{
  "graph": {
    "orphan_entry_patterns": [
      "src/controllers/**/*.ts",
      "src/commands/**/*.ts",
      "tests/**/*"
    ]
  }
}</code></pre>

<h2 id="real-orphans">Identificar huerfanos reales</h2>
<p>Despues de excluir puntos de entrada, los huerfanos restantes incluyen: archivos de funcionalidades obsoletas, implementaciones incompletas, utilidades copiadas y stubs generados.</p>

<h2 id="orphans-in-monorepos">Huerfanos en monorepos</h2>
<p>En monorepos, la deteccion de huerfanos es especialmente valiosa. Los paquetes que ningun otro paquete importa pueden ser miembros del workspace completamente sin usar.</p>`,
      zh: `<h2 id="what-are-orphan-files">什么是孤立文件？</h2>
<p>孤立文件是项目中没有其他文件导入的文件。它在依赖图中有零个入站依赖。这些连接网络之外的文件要么是可以删除的死代码，要么是由依赖图之外的东西（如CLI运行器、Web框架或测试框架）调用的入口点。</p>

<h2 id="how-detection-works">检测原理</h2>
<ol>
<li><strong>构建图</strong> — 使用petgraph构建所有文件及其导入关系的有向图。</li>
<li><strong>计算入度</strong> — 对每个文件计算入边数。入度为0的文件是孤立候选者。</li>
<li><strong>应用入口点模式</strong> — 将每个候选者与 <code>policy.json</code> 中配置的入口点模式进行匹配。</li>
<li><strong>报告剩余孤立文件</strong> — 不匹配任何入口点模式的文件被报告为孤立文件。</li>
</ol>

<h2 id="entry-point-patterns">入口点模式</h2>
<p>入口点是外部调用的文件：控制器、CLI命令、迁移、引导文件、配置文件和测试文件。配置入口点模式：</p>
<pre><code>{
  "graph": {
    "orphan_entry_patterns": [
      "src/controllers/**/*.ts",
      "src/commands/**/*.ts",
      "tests/**/*"
    ]
  }
}</code></pre>

<h2 id="real-orphans">识别真正的孤立文件</h2>
<p>排除入口点后，剩余的孤立文件包括：已弃用功能的文件、未完成的实现、复制的工具文件和生成的存根。</p>

<h2 id="orphans-in-monorepos">Monorepo中的孤立文件</h2>
<p>在monorepo中，孤立文件检测特别有价值。没有其他包导入的包可能是完全未使用的工作区成员。</p>`,
      hi: `<h2 id="what-are-orphan-files">ऑर्फ़न फ़ाइलें क्या हैं?</h2>
<p>ऑर्फ़न फ़ाइल वह फ़ाइल है जिसे आपके प्रोजेक्ट में कोई अन्य फ़ाइल इम्पोर्ट नहीं करती। डिपेंडेंसी ग्राफ में इसकी शून्य इनबाउंड डिपेंडेंसी होती है। कनेक्शन के इस जाल के बाहर की फ़ाइलें या तो डेड कोड हैं जिन्हें हटाया जा सकता है, या डिपेंडेंसी ग्राफ के बाहर किसी चीज (जैसे CLI रनर, वेब फ्रेमवर्क या टेस्ट हार्नेस) द्वारा इनवोक किए जाने वाले एंट्री पॉइंट हैं।</p>

<h2 id="how-detection-works">डिटेक्शन कैसे काम करता है</h2>
<ol>
<li><strong>ग्राफ बनाएं</strong> — petgraph का उपयोग करके सभी फ़ाइलों और उनके इम्पोर्ट रिलेशनशिप का डायरेक्टेड ग्राफ बनाया जाता है।</li>
<li><strong>इन-डिग्री की गणना</strong> — प्रत्येक फ़ाइल के लिए इनकमिंग एज गिनें। इन-डिग्री 0 वाली फ़ाइलें ऑर्फ़न कैंडिडेट हैं।</li>
<li><strong>एंट्री पॉइंट पैटर्न लागू करें</strong> — प्रत्येक कैंडिडेट को <code>policy.json</code> में कॉन्फ़िगर किए गए पैटर्न से मैच करें।</li>
<li><strong>शेष ऑर्फ़न रिपोर्ट करें</strong> — किसी भी पैटर्न से मैच नहीं करने वाली फ़ाइलें ऑर्फ़न के रूप में रिपोर्ट की जाती हैं।</li>
</ol>

<h2 id="entry-point-patterns">एंट्री पॉइंट पैटर्न</h2>
<p>एंट्री पॉइंट बाहरी रूप से इनवोक की जाने वाली फ़ाइलें हैं: कंट्रोलर, CLI कमांड, माइग्रेशन, बूटस्ट्रैप फ़ाइलें, कॉन्फ़िगरेशन फ़ाइलें और टेस्ट फ़ाइलें। पैटर्न कॉन्फ़िगर करें:</p>
<pre><code>{
  "graph": {
    "orphan_entry_patterns": [
      "src/controllers/**/*.ts",
      "src/commands/**/*.ts",
      "tests/**/*"
    ]
  }
}</code></pre>

<h2 id="real-orphans">वास्तविक ऑर्फ़न की पहचान</h2>
<p>एंट्री पॉइंट को बाहर करने के बाद, शेष ऑर्फ़न में शामिल हैं: बंद की गई सुविधाओं की फ़ाइलें, अधूरे कार्यान्वयन, कॉपी की गई यूटिलिटी और जनरेट किए गए स्टब।</p>

<h2 id="orphans-in-monorepos">मोनोरेपो में ऑर्फ़न</h2>
<p>मोनोरेपो में, ऑर्फ़न डिटेक्शन विशेष रूप से मूल्यवान है। जिन पैकेज को कोई अन्य पैकेज इम्पोर्ट नहीं करता, वे पूरी तरह से अप्रयुक्त वर्कस्पेस सदस्य हो सकते हैं।</p>`,
      pt: `<h2 id="what-are-orphan-files">O que sao arquivos orfaos?</h2>
<p>Um arquivo orfao e um arquivo que nenhum outro arquivo do projeto importa. Tem zero dependencias de entrada no grafo. Arquivos fora desta rede de conexoes sao codigo morto que pode ser deletado, ou pontos de entrada invocados por algo fora do grafo (como um runner CLI, framework web ou harness de teste).</p>

<h2 id="how-detection-works">Como a deteccao funciona</h2>
<ol>
<li><strong>Construir o grafo</strong> — Um grafo direcionado de todos os arquivos e suas relacoes de importacao e construido com petgraph.</li>
<li><strong>Calcular grau de entrada</strong> — Para cada arquivo, contar arestas de entrada. Arquivos com grau 0 sao candidatos orfaos.</li>
<li><strong>Aplicar padroes de entrada</strong> — Cada candidato e verificado contra os padroes configurados no <code>policy.json</code>.</li>
<li><strong>Reportar orfaos restantes</strong> — Arquivos que nao correspondem a nenhum padrao sao reportados como orfaos.</li>
</ol>

<h2 id="entry-point-patterns">Padroes de pontos de entrada</h2>
<p>Pontos de entrada sao arquivos invocados externamente: controllers, comandos CLI, migracoes, arquivos de bootstrap, arquivos de configuracao e arquivos de teste. Configure os padroes:</p>
<pre><code>{
  "graph": {
    "orphan_entry_patterns": [
      "src/controllers/**/*.ts",
      "src/commands/**/*.ts",
      "tests/**/*"
    ]
  }
}</code></pre>

<h2 id="real-orphans">Identificar orfaos reais</h2>
<p>Apos excluir pontos de entrada, os orfaos restantes incluem: arquivos de funcionalidades obsoletas, implementacoes incompletas, utilitarios copiados e stubs gerados.</p>

<h2 id="orphans-in-monorepos">Orfaos em monorepos</h2>
<p>Em monorepos, a deteccao de orfaos e particularmente valiosa. Pacotes que nenhum outro pacote importa podem ser membros do workspace completamente nao utilizados.</p>`,
      ar: `<h2 id="what-are-orphan-files">ما هي الملفات اليتيمة؟</h2>
<p>الملف اليتيم هو ملف لا يستورد منه أي ملف آخر في مشروعك. لديه صفر تبعيات واردة في رسم التبعيات البياني. قد يكون شيفرة ميتة أو نقطة دخول مشروعة مثل سكربت CLI أو مشغّل اختبارات أو ملف ترحيل.</p>

<h2 id="identifying-real-orphans">تحديد الملفات اليتيمة الحقيقية</h2>
<p>يستخدم AigisCode أنماطاً مدفوعة بالسياسات للتمييز بين الملفات اليتيمة الحقيقية ونقاط الدخول. تشمل نقاط الدخول الشائعة:</p>
<ul>
<li><strong>سكربتات CLI</strong> — ملفات تُنفّذ مباشرة من سطر الأوامر.</li>
<li><strong>ملفات التكوين</strong> — إعدادات التطبيق وتكوين الإطار.</li>
<li><strong>ملفات الترحيل</strong> — ترحيلات قاعدة البيانات التي يُنفّذها الإطار.</li>
<li><strong>ملفات الاختبار</strong> — اختبارات تُشغّل بواسطة أداة تشغيل الاختبارات.</li>
</ul>

<h2 id="impact">تأثير إزالة الملفات اليتيمة</h2>
<ul>
<li><strong>حجم حزمة أصغر</strong> — ملفات أقل تعني حزم أصغر وبناء أسرع.</li>
<li><strong>بحث أسرع</strong> — نتائج بحث أقل ضوضاءً ومراجعات شيفرة أسهل.</li>
<li><strong>تأهيل أسرع</strong> — المطورون الجدد لديهم شيفرة أقل لفهمها.</li>
<li><strong>سطح أمني أصغر</strong> — شيفرة أقل تعني نقاط ضعف محتملة أقل.</li>
</ul>`,
      pl: `<h2 id="orphan-files">Osierocone pliki</h2>
<p>AigisCode identyfikuje pliki bez przychodzacych zaleznosci. Pliki pasujace do wzorow punktow wejscia sa wyklaczane.</p>`,
    },
    capabilities: [
      'Zero inbound dependency detection',
      'Policy-driven entry point patterns',
      'Controller/command/migration exclusion',
      'Bootstrap file recognition',
      'Cross-package orphan detection in monorepos',
      'In-degree graph analysis',
      'Confidence scoring',
      'Complements dead code detection',
    ],
    codeExample: `# Analyze for orphan files
aigiscode analyze /path/to/project

# View orphan files
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.orphan_files'

# Configure entry point patterns
cat > .aigiscode/policy.json << EOF
{
  "graph": {
    "orphan_entry_patterns": [
      "src/controllers/**/*.ts",
      "src/commands/**/*.ts",
      "tests/**/*"
    ]
  }
}
EOF`,
    relatedSlugs: ['dead-code', 'bottlenecks', 'circular-dependencies'],
  },
];

/* -------------------------------------------------------------------------- */
/*  Derived data                                                              */
/* -------------------------------------------------------------------------- */

export const categories = [...new Set(features.map((f) => f.category))];

/* -------------------------------------------------------------------------- */
/*  Helpers                                                                   */
/* -------------------------------------------------------------------------- */

export function getFeatureBySlug(slug: string): Feature | undefined {
  return features.find((f) => f.slug === slug);
}

export function getRelatedFeatures(slugs: string[]): Feature[] {
  return slugs
    .map((s) => features.find((f) => f.slug === s))
    .filter(Boolean) as Feature[];
}
