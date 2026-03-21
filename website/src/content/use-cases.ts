/* -------------------------------------------------------------------------- */
/*  Use Case Data — AigisCode Marketing Website                              */
/* -------------------------------------------------------------------------- */

export interface UseCase {
  slug: string;
  icon: string;
  category: string;
  title: Record<string, string>;
  shortDescription: Record<string, string>;
  metaDescription: Record<string, string>;
  content: Record<string, string>;
  features: string[];
  codeExample?: string;
  relatedSlugs: string[];
}

export const useCases: UseCase[] = [
  /* ---- 1. Circular Dependency Detection ---- */
  {
    slug: 'circular-dependency-detection',
    icon: 'ArrowsClockwise',
    category: 'Detection',
    title: {
      en: 'Circular Dependency Detection',
      zh: '循环依赖检测',
      hi: 'सर्कुलर डिपेंडेंसी डिटेक्शन',
      pt: 'Detecção de Dependências Circulares',
      ar: 'اكتشاف التبعيات الدائرية',
      pl: 'Wykrywanie cyklicznych zaleznosci',
      bn: 'সার্কুলার ডিপেন্ডেন্সি ডিটেকশন',
      fr: 'Détection des Dépendances Circulaires',
      es: 'Detección de Dependencias Circulares',
      cs: 'Detekce Cyklických Závislostí',
    },
    shortDescription: {
      en: 'Find architectural cycles between modules that create tight coupling and make refactoring dangerous. AigisCode distinguishes strong architectural cycles from benign runtime load-order dependencies.',
      zh: '查找模块间的架构循环，这些循环会造成紧耦合并使重构变得危险。',
      hi: 'मॉड्यूल के बीच आर्किटेक्चरल साइकिल खोजें जो टाइट कपलिंग बनाते हैं।',
      pt: 'Encontre ciclos arquiteturais entre módulos que criam acoplamento forte e tornam a refatoração perigosa.',
      ar: 'اعثر على الدورات المعمارية بين الوحدات التي تخلق اقتراناً محكماً وتجعل إعادة البناء خطيرة. يميّز AigisCode الدورات المعمارية القوية عن تبعيات ترتيب التحميل الحميدة.',
      pl: 'Znajdz architektoniczne cykle miedzy modulami, ktore tworza scisle powiazania i utrudniaja refaktoryzacje. AigisCode rozroznia silne cykle architektoniczne od nieszkodliwych zaleznosci kolejnosci ladowania.',
      bn: 'মডিউলগুলোর মধ্যে আর্কিটেকচারাল চক্র খুঁজুন যা টাইট কাপলিং তৈরি করে এবং রিফ্যাক্টরিং ঝুঁকিপূর্ণ করে। AigisCode শক্তিশালী আর্কিটেকচারাল চক্রকে সাধারণ রানটাইম লোড-অর্ডার ডিপেন্ডেন্সি থেকে আলাদা করে।',
      fr: 'Trouvez les cycles architecturaux entre modules qui créent un couplage fort et rendent le refactoring dangereux.',
      es: 'Encuentre ciclos arquitectónicos entre módulos que crean acoplamiento fuerte y hacen peligrosa la refactorización.',
      cs: 'Najděte architektonické cykly mezi moduly, které vytvářejí těsné vazby a činí refaktoring nebezpečným.',
    },
    metaDescription: {
      en: 'Detect circular dependencies in PHP, Python, TypeScript, and JavaScript codebases. AigisCode finds strong architectural cycles that single-file linters miss.',
      zh: '在PHP、Python、TypeScript和JavaScript代码库中检测循环依赖。',
      hi: 'PHP, Python, TypeScript और JavaScript कोडबेस में सर्कुलर डिपेंडेंसी का पता लगाएं।',
      pt: 'Detecte dependências circulares em codebases PHP, Python, TypeScript e JavaScript.',
      ar: 'اكتشف التبعيات الدائرية في قواعد شيفرة PHP وPython وTypeScript وJavaScript. يجد AigisCode الدورات المعمارية القوية التي تفوتها أدوات فحص الملف الواحد.',
      pl: 'Wykrywaj cykliczne zaleznosci w bazach kodu PHP, Python, TypeScript i JavaScript. AigisCode znajduje silne cykle architektoniczne, ktore jednorazowe lintery pomijaja.',
      bn: 'PHP, Python, TypeScript এবং JavaScript কোডবেসে সার্কুলার ডিপেন্ডেন্সি শনাক্ত করুন। AigisCode সিঙ্গেল-ফাইল লিন্টার মিস করে এমন শক্তিশালী আর্কিটেকচারাল চক্র খুঁজে বের করে।',
      fr: 'Détectez les dépendances circulaires dans les codebases PHP, Python, TypeScript et JavaScript.',
      es: 'Detecte dependencias circulares en bases de código PHP, Python, TypeScript y JavaScript.',
      cs: 'Detekujte cyklické závislosti v kódových bázích PHP, Python, TypeScript a JavaScript.',
    },
    content: {
      en: `<h2 id="what-are-circular-dependencies">What Are Circular Dependencies?</h2>
<p>Circular dependencies occur when two or more modules depend on each other, directly or indirectly. Module A imports Module B, which imports Module C, which imports Module A — creating a cycle. These cycles are one of the most insidious architectural problems in modern codebases because they silently erode modularity, make testing painful, and can lead to unpredictable initialization behavior.</p>
<p>Most single-file linters cannot detect circular dependencies because the problem only becomes visible when you analyze the <strong>entire dependency graph</strong> of a project. A file in isolation looks perfectly fine. It is only when you trace the import chain across dozens of files that cycles emerge.</p>

<h2 id="strong-vs-runtime-cycles">Strong Cycles vs. Runtime Cycles</h2>
<p>AigisCode distinguishes between two types of circular dependencies:</p>
<ul>
<li><strong>Strong architectural cycles</strong> — These are cycles in the static import graph where modules are structurally coupled. Refactoring one module in the cycle forces changes to all others. These are high-priority findings.</li>
<li><strong>Runtime load-order cycles</strong> — These occur when modules have circular imports that are resolved at runtime through lazy loading or deferred access. They are lower priority but still worth tracking.</li>
</ul>
<p>This distinction matters because blindly flagging every cycle creates noise. A Django project might have models that reference each other through string-based foreign keys — technically a cycle in the import graph, but not a structural problem. AigisCode's graph analyzer uses strongly connected component (SCC) analysis to find the architectural cycles that actually hurt your codebase.</p>

<h2 id="how-aigiscode-detects-cycles">How AigisCode Detects Cycles</h2>
<p>The detection process works in three stages:</p>
<ol>
<li><strong>Index</strong> — AigisCode parses every file using tree-sitter (for PHP, TypeScript, JavaScript, Vue) or tree-sitter-python, extracting all import statements and symbol references.</li>
<li><strong>Graph</strong> — A file-level dependency graph is built using petgraph. Import aliases (like <code>@/</code> mapping to <code>src/</code>) are resolved using your policy configuration.</li>
<li><strong>Detect</strong> — The graph analyzer runs Tarjan's algorithm to find strongly connected components. Components with more than one node represent architectural cycles.</li>
</ol>

<h2 id="real-world-example">Real-World Example</h2>
<p>Consider a typical Express.js application where the route handlers import a validation middleware, the validation middleware imports the database models, and the database models import utility functions that reference the route configuration:</p>
<pre><code>// routes/users.ts → middleware/validate.ts → models/User.ts → utils/config.ts → routes/index.ts
// This creates a 4-node cycle that makes it impossible to test any single module in isolation.</code></pre>
<p>AigisCode would report this as a strong circular dependency with all participating files listed. The report includes the full cycle path so you can decide where to break the chain — typically by extracting shared types into a separate module or inverting a dependency through an interface.</p>

<h2 id="fixing-circular-dependencies">Strategies for Breaking Cycles</h2>
<p>Once AigisCode identifies your cycles, common resolution strategies include:</p>
<ul>
<li><strong>Extract shared types</strong> — Move interfaces and type definitions into a dedicated <code>types/</code> or <code>contracts/</code> directory that has no outbound dependencies.</li>
<li><strong>Dependency inversion</strong> — Instead of Module A importing Module B directly, define an interface in Module A that Module B implements. This inverts the dependency direction.</li>
<li><strong>Lazy imports</strong> — In Python, move imports inside functions. In TypeScript, use dynamic <code>import()</code>. This breaks the static cycle but preserves runtime access.</li>
<li><strong>Mediator pattern</strong> — Introduce a third module that both A and B depend on, eliminating the direct cycle.</li>
</ul>

<h2 id="running-the-analysis">Running the Analysis</h2>
<p>Run AigisCode on your project and check the <code>graph_analysis.strong_circular_dependencies</code> field in the JSON report. Each entry includes the cycle members, the edge list, and a confidence score. Start with the smallest cycles — they are usually the easiest to fix and deliver the biggest architectural improvement per effort.</p>`,
      zh: `<h2 id="what-are-circular-dependencies">什么是循环依赖？</h2>
<p>循环依赖发生在两个或更多模块直接或间接地相互依赖时。AigisCode使用Tarjan算法检测强连通分量，区分强架构循环和运行时加载顺序循环。强架构循环是高优先级发现，因为它们会造成紧耦合并使重构变得危险。</p>
<h2 id="how-it-works">工作原理</h2>
<p>检测过程分三个阶段：索引（解析所有导入语句）、图构建（使用petgraph构建文件级依赖图）和检测（运行SCC分析）。报告包括完整的循环路径，以便您决定在哪里断开链条。</p>`,
      fr: `<h2 id="what-are-circular-dependencies">Que sont les dépendances circulaires ?</h2>
<p>Les dépendances circulaires surviennent lorsque deux modules ou plus dépendent les uns des autres. AigisCode utilise l'algorithme de Tarjan pour détecter les composants fortement connectés, distinguant les cycles architecturaux forts des cycles d'ordre de chargement à l'exécution. Exécutez l'analyse et consultez le champ <code>graph_analysis.strong_circular_dependencies</code> dans le rapport JSON.</p>`,
      es: `<h2 id="what-are-circular-dependencies">¿Qué son las dependencias circulares?</h2>
<p>Las dependencias circulares ocurren cuando dos o más módulos dependen entre sí. AigisCode utiliza el algoritmo de Tarjan para detectar componentes fuertemente conectados, distinguiendo ciclos arquitectónicos fuertes de ciclos de orden de carga en tiempo de ejecución. Ejecute el análisis y revise el campo <code>graph_analysis.strong_circular_dependencies</code> en el informe JSON.</p>`,
      cs: `<h2 id="what-are-circular-dependencies">Co jsou cyklické závislosti?</h2>
<p>Cyklické závislosti vznikají, když dva nebo více modulů závisí na sobě navzájem. AigisCode používá Tarjanův algoritmus k detekci silně propojených komponent a rozlišuje silné architektonické cykly od běhových cyklů pořadí načítání. Spusťte analýzu a zkontrolujte pole <code>graph_analysis.strong_circular_dependencies</code> v JSON reportu.</p>`,
      hi: `<h2 id="what-are-circular-dependencies">सर्कुलर डिपेंडेंसी क्या हैं?</h2>
<p>सर्कुलर डिपेंडेंसी तब होती है जब दो या अधिक मॉड्यूल एक-दूसरे पर निर्भर करते हैं। AigisCode स्ट्रॉन्गली कनेक्टेड कंपोनेंट का पता लगाने के लिए Tarjan एल्गोरिदम का उपयोग करता है, स्ट्रॉन्ग आर्किटेक्चरल साइकिल को रनटाइम लोड-ऑर्डर साइकिल से अलग करता है।</p>
<h2 id="how-it-works">कैसे काम करता है</h2>
<p>डिटेक्शन तीन चरणों में काम करता है: इंडेक्स (सभी इम्पोर्ट स्टेटमेंट पार्स करना), ग्राफ़ (petgraph से फ़ाइल-लेवल डिपेंडेंसी ग्राफ़ बनाना) और डिटेक्ट (SCC एनालिसिस चलाना)। रिपोर्ट में पूरा साइकिल पाथ शामिल है।</p>`,
      pt: `<h2 id="what-are-circular-dependencies">O que sao dependencias circulares?</h2>
<p>Dependencias circulares ocorrem quando dois ou mais modulos dependem um do outro. O AigisCode usa o algoritmo de Tarjan para detectar componentes fortemente conectados, distinguindo ciclos arquiteturais fortes de ciclos de ordem de carga em tempo de execucao.</p>
<h2 id="how-it-works">Como funciona</h2>
<p>A deteccao funciona em tres etapas: indice (analise de todas as declaracoes de import), grafo (construcao do grafo de dependencias com petgraph) e deteccao (execucao da analise SCC). O relatorio inclui o caminho completo do ciclo.</p>`,
      ar: `<h2 id="what-are-circular-dependencies">ما هي التبعيات الدائرية؟</h2>
<p>تحدث التبعيات الدائرية عندما تعتمد وحدتان أو أكثر على بعضها البعض مما يخلق دورة في رسم التبعيات البياني. تآكل هذه الدورات النمطية بصمت وتجعل إعادة البناء خطيرة وتؤدي إلى فشل تهيئة غير متوقع.</p>
<p>يميّز AigisCode الدورات المعمارية القوية (اقتران هيكلي يحتاج إعادة بناء) عن دورات وقت التشغيل (تُحل عبر التحميل الكسول). يستخدم خوارزمية Tarjan SCC عبر petgraph للعثور على المكونات المترابطة بقوة.</p>`,
      pl: `<h2 id="what-are-circular-dependencies">Czym sa cykliczne zaleznosci?</h2>
<p>AigisCode uzywa algorytmu Tarjana do wykrywania silnie spojnych komponentow. Rozroznia silne cykle architektoniczne od cykli runtime. Raport zawiera pelna sciezke cyklu.</p>`,
    },
    features: [
      'Tarjan SCC analysis',
      'Strong vs. runtime cycle distinction',
      'Multi-language support',
      'Full cycle path reporting',
      'petgraph graph engine',
      'Policy-aware alias resolution',
    ],
    codeExample: `# Analyze your project for circular dependencies
aigiscode analyze /path/to/project

# Check the report
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.strong_circular_dependencies'`,
    relatedSlugs: ['dead-code-detection', 'typescript-monorepo-analysis', 'ci-cd-integration'],
  },
  /* ---- 2. Dead Code Detection ---- */
  {
    slug: 'dead-code-detection',
    icon: 'Trash',
    category: 'Detection',
    title: {
      en: 'Dead Code Detection',
      zh: '死代码检测',
      hi: 'डेड कोड डिटेक्शन',
      pt: 'Detecção de Código Morto',
      ar: 'اكتشاف الشيفرة الميتة',
      pl: 'Wykrywanie martwego kodu',
      bn: 'ডেড কোড ডিটেকশন',
      fr: 'Détection de Code Mort',
      es: 'Detección de Código Muerto',
      cs: 'Detekce Mrtvého Kódu',
    },
    shortDescription: {
      en: 'Eliminate unused imports, abandoned classes, unreachable methods, and orphaned files. AigisCode traces symbol references across your entire project to find code that nothing calls.',
      zh: '消除未使用的导入、废弃的类、不可达的方法和孤立文件。',
      hi: 'अनुपयोगी इम्पोर्ट, छोड़ी गई क्लासेज, अनपहुंच मेथड्स और अनाथ फाइलें खोजें।',
      pt: 'Elimine imports não utilizados, classes abandonadas, métodos inacessíveis e arquivos órfãos.',
      ar: 'أزل الاستيرادات غير المستخدمة والفئات المهجورة والطرق غير القابلة للوصول والملفات اليتيمة. يتتبع AigisCode مراجع الرموز عبر مشروعك بالكامل للعثور على الشيفرة التي لا يستدعيها شيء.',
      pl: 'Eliminuj nieuzywane importy, porzucone klasy, nieosiagalne metody i osierocone pliki. AigisCode sledzi odwolania do symboli w calym projekcie, aby znalezc kod, ktory nic nie wywoluje.',
      bn: 'অব্যবহৃত ইমপোর্ট, পরিত্যক্ত ক্লাস, অপ্রাপ্য মেথড এবং অনাথ ফাইল নির্মূল করুন। AigisCode আপনার সম্পূর্ণ প্রজেক্ট জুড়ে সিম্বল রেফারেন্স ট্রেস করে এমন কোড খুঁজে যা কিছুই কল করে না।',
      fr: 'Eliminez les imports inutilisés, les classes abandonnées, les méthodes inaccessibles et les fichiers orphelins.',
      es: 'Elimine imports no utilizados, clases abandonadas, métodos inalcanzables y archivos huérfanos.',
      cs: 'Odstraňte nepoužívané importy, opuštěné třídy, nedosažitelné metody a osiřelé soubory.',
    },
    metaDescription: {
      en: 'Find and remove dead code in PHP, Python, TypeScript, and JavaScript projects. AigisCode detects unused imports, abandoned classes, orphan files, and unreferenced methods.',
      zh: '在PHP、Python、TypeScript和JavaScript项目中查找和删除死代码。',
      hi: 'PHP, Python, TypeScript और JavaScript प्रोजेक्ट्स में डेड कोड खोजें और हटाएं।',
      pt: 'Encontre e remova código morto em projetos PHP, Python, TypeScript e JavaScript.',
      ar: 'اعثر على الشيفرة الميتة وأزلها في مشاريع PHP وPython وTypeScript وJavaScript. يكتشف AigisCode الاستيرادات غير المستخدمة والفئات المهجورة والملفات اليتيمة والطرق غير المرجعية.',
      pl: 'Znajdz i usun martwy kod w projektach PHP, Python, TypeScript i JavaScript. AigisCode wykrywa nieuzywane importy, porzucone klasy, osierocone pliki i nieodwolywane metody.',
      bn: 'PHP, Python, TypeScript এবং JavaScript প্রজেক্টে ডেড কোড খুঁজুন এবং সরান। AigisCode অব্যবহৃত ইমপোর্ট, পরিত্যক্ত ক্লাস, অনাথ ফাইল এবং অরেফারেন্সড মেথড শনাক্ত করে।',
      fr: 'Trouvez et supprimez le code mort dans les projets PHP, Python, TypeScript et JavaScript.',
      es: 'Encuentre y elimine código muerto en proyectos PHP, Python, TypeScript y JavaScript.',
      cs: 'Najděte a odstraňte mrtvý kód v projektech PHP, Python, TypeScript a JavaScript.',
    },
    content: {
      en: `<h2 id="the-cost-of-dead-code">The Hidden Cost of Dead Code</h2>
<p>Dead code is the silent tax every growing codebase pays. Unused imports inflate bundle sizes. Abandoned classes confuse new developers who waste hours understanding code that nothing actually calls. Orphaned files accumulate in directories, making searches slower and code reviews harder. Over time, dead code becomes a breeding ground for security vulnerabilities — code that nobody maintains is code that nobody patches.</p>
<p>The challenge is that dead code is nearly invisible at the file level. An individual import looks fine. A class might appear to be used because it follows a naming convention. Only when you analyze the <strong>entire dependency graph</strong> does the dead code reveal itself: no inbound edges in the graph means no references, which means dead code.</p>

<h2 id="what-aigiscode-finds">What AigisCode Finds</h2>
<p>AigisCode's dead code detector identifies four categories of unused code:</p>
<ul>
<li><strong>Unused imports</strong> — Import statements that bring in symbols never referenced in the file. This is the most common type of dead code and the easiest to fix.</li>
<li><strong>Abandoned classes</strong> — Classes that exist in the codebase but are never instantiated, extended, or referenced by any other file. These often result from incomplete feature removals.</li>
<li><strong>Unreferenced methods</strong> — Public methods on classes that are never called from outside the class. While they might be used through reflection or dynamic dispatch, AigisCode flags them with confidence levels so you can triage appropriately.</li>
<li><strong>Orphan files</strong> — Files with zero inbound dependencies in the import graph. No other file imports from them, no configuration references them, and no entry point includes them.</li>
</ul>

<h2 id="how-detection-works">How the Detection Works</h2>
<p>Dead code detection runs during the Detect stage of AigisCode's six-stage pipeline:</p>
<ol>
<li><strong>Symbol extraction</strong> — The indexer parses every file to extract all exported symbols: classes, functions, constants, type definitions, and re-exports.</li>
<li><strong>Reference tracing</strong> — The graph builder tracks every import statement to determine which symbols are actually consumed by other files.</li>
<li><strong>Orphan analysis</strong> — Files with zero inbound edges in the dependency graph are flagged as orphans. Entry point patterns from your policy file are excluded.</li>
<li><strong>Confidence scoring</strong> — Each finding gets a confidence level. Unused imports are high confidence. Abandoned classes in a framework with dependency injection are medium confidence because the DI container might reference them dynamically.</li>
</ol>

<h2 id="real-world-scenario">Real-World Scenario</h2>
<p>Consider a TypeScript monorepo with 500 files. Over 18 months of development, 40 files were created for features that were later removed or replaced. The old files still exist because nobody was sure if something still depended on them. Running AigisCode reveals:</p>
<pre><code>$ aigiscode analyze .
  Dead Code Results:
  ├── 23 unused imports across 15 files
  ├── 7 abandoned classes (0 references each)
  ├── 12 orphan files (no inbound dependencies)
  └── 5 unreferenced public methods</code></pre>
<p>The report provides the exact file paths and symbol names, along with confidence scores. A developer can review the findings, confirm each one, and remove the dead code in a single focused PR — immediately reducing cognitive load and improving build times.</p>

<h2 id="policy-customization">Policy Customization</h2>
<p>Not all code without explicit references is truly dead. Entry points like CLI scripts, migration files, test fixtures, and service provider registrations are valid code that simply is not imported by other files. AigisCode's policy system lets you specify entry point patterns:</p>
<pre><code>{
  "dead_code": {
    "abandoned_entry_patterns": [
      "src/commands/**/*.ts",
      "database/migrations/**/*.php",
      "tests/fixtures/**/*"
    ]
  }
}</code></pre>
<p>With this policy, files matching those patterns will not be flagged as orphans, reducing false positives and focusing the report on genuinely unused code.</p>

<h2 id="impact">Measurable Impact</h2>
<p>Teams that regularly run dead code detection with AigisCode typically see bundle size reductions of 5-15%, faster IDE indexing, shorter code review cycles, and a measurable decrease in onboarding time for new developers. Dead code removal is one of the highest-ROI maintenance tasks you can perform.</p>`,
      zh: `<h2 id="the-cost-of-dead-code">死代码的隐藏成本</h2>
<p>死代码是每个增长中的代码库都要付出的无形税收。未使用的导入增大包体积，废弃的类让新开发者困惑，孤立文件使搜索变慢。AigisCode检测四类未使用代码：未使用的导入、废弃的类、未引用的方法和孤立文件。</p>
<h2 id="how-it-works">工作原理</h2>
<p>通过符号提取、引用追踪和孤立分析三个步骤，AigisCode在整个依赖图中追踪符号引用。通过策略文件自定义入口点模式以减少误报。</p>`,
      fr: `<h2 id="the-cost-of-dead-code">Le coût caché du code mort</h2>
<p>Le code mort est la taxe silencieuse que paie chaque base de code en croissance. AigisCode détecte quatre catégories : imports inutilisés, classes abandonnées, méthodes non référencées et fichiers orphelins. L'analyseur trace les références de symboles à travers l'ensemble du graphe de dépendances pour trouver le code que rien n'appelle.</p>`,
      es: `<h2 id="the-cost-of-dead-code">El costo oculto del código muerto</h2>
<p>El código muerto es el impuesto silencioso que paga cada base de código en crecimiento. AigisCode detecta cuatro categorías: imports no utilizados, clases abandonadas, métodos no referenciados y archivos huérfanos. El analizador rastrea referencias de símbolos a través de todo el grafo de dependencias.</p>`,
      cs: `<h2 id="the-cost-of-dead-code">Skryté náklady mrtvého kódu</h2>
<p>Mrtvý kód je tichá daň, kterou platí každá rostoucí kódová báze. AigisCode detekuje čtyři kategorie: nepoužívané importy, opuštěné třídy, neodkazované metody a osiřelé soubory. Analyzátor sleduje reference symbolů napříč celým grafem závislostí.</p>`,
      hi: `<h2 id="the-cost-of-dead-code">डेड कोड की छिपी लागत</h2>
<p>डेड कोड हर बढ़ते कोडबेस का साइलेंट टैक्स है। AigisCode चार कैटेगरी डिटेक्ट करता है: अनुपयोगी इम्पोर्ट, एबंडन्ड क्लासेस, अनरेफ़रेंस्ड मेथड और ऑर्फ़न फ़ाइलें। एनालाइज़र पूरे डिपेंडेंसी ग्राफ़ में सिंबल रेफ़रेंस ट्रेस करता है।</p>`,
      pt: `<h2 id="the-cost-of-dead-code">O custo oculto do codigo morto</h2>
<p>Codigo morto e o imposto silencioso que toda base de codigo em crescimento paga. O AigisCode detecta quatro categorias: imports nao utilizados, classes abandonadas, metodos nao referenciados e arquivos orfaos. O analisador rastrea referencias de simbolos em todo o grafo de dependencias.</p>`,
      ar: `<h2 id="hidden-cost">التكلفة الخفية للشيفرة الميتة</h2>
<p>تتراكم الشيفرة الميتة بصمت: استيرادات غير مستخدمة وفئات مهجورة وملفات يتيمة. تزيد العبء المعرفي وتبطئ البناء وتوسّع سطح الأمان. يتتبع AigisCode مراجع الرموز عبر المشروع بالكامل للعثور على الشيفرة التي لا يستدعيها شيء.</p>`,
      pl: `<h2 id="dead-code">Martwy kod</h2>
<p>AigisCode sledzi odwolania do symboli. Identyfikuje nieuzywane importy, porzucone klasy, nieodwolywane metody i osierocone pliki z poziomami pewnosci. Wzorce punktow wejscia mozna zdefiniowac w politykach.</p>`,
    },
    features: [
      'Unused import detection',
      'Abandoned class finder',
      'Orphan file analysis',
      'Unreferenced method tracking',
      'Confidence scoring',
      'Entry point exclusion patterns',
    ],
    codeExample: `# Run dead code analysis
aigiscode analyze /path/to/project

# View dead code findings
cat .aigiscode/deterministic-analysis.json | jq '.dead_code'

# Check orphan files specifically
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.orphan_files'`,
    relatedSlugs: ['circular-dependency-detection', 'python-codebase-analysis', 'typescript-monorepo-analysis'],
  },
  /* ---- 3. Hardwired Values Detection ---- */
  {
    slug: 'hardwired-values-detection',
    icon: 'LinkBreak',
    category: 'Detection',
    title: {
      en: 'Hardwired Values Detection',
      zh: '硬编码值检测',
      hi: 'हार्डवायर्ड वैल्यूज डिटेक्शन',
      pt: 'Detecção de Valores Hardcoded',
      ar: 'اكتشاف القيم الثابتة',
      pl: 'Wykrywanie zakodowanych wartosci',
      bn: 'হার্ডওয়্যার্ড ভ্যালু ডিটেকশন',
      fr: 'Détection des Valeurs en Dur',
      es: 'Detección de Valores Hardcodeados',
      cs: 'Detekce Natvrdo Zadaných Hodnot',
    },
    shortDescription: {
      en: 'Catch magic strings, repeated literals, hardcoded IPs, URLs, and environment variable access outside configuration files. AigisCode finds the hardwired values that create maintenance nightmares.',
      zh: '捕获魔法字符串、重复字面量、硬编码IP、URL以及配置文件外的环境变量访问。',
      hi: 'मैजिक स्ट्रिंग्स, रिपीटेड लिटरल्स, हार्डकोडेड IPs और कॉन्फ़िग फाइलों के बाहर एनवायरनमेंट वेरिएबल एक्सेस पकड़ें।',
      pt: 'Capture strings mágicas, literais repetidos, IPs hardcoded e acesso a variáveis de ambiente fora dos arquivos de configuração.',
      ar: 'اكتشف السلاسل السحرية والقيم الحرفية المتكررة وعناوين IP وURLs الثابتة والوصول لمتغيرات البيئة خارج ملفات التكوين. يجد AigisCode القيم الثابتة التي تخلق كوابيس الصيانة.',
      pl: 'Wychwytuj magiczne stringi, powtarzajace sie literaly, zakodowane adresy IP, URL-e i dostep do zmiennych srodowiskowych poza plikami konfiguracyjnymi. Detektor AigisCode respektuje wylaczenia polityk, wiec wzorce frameworkowe nie sa oznaczane.',
      bn: 'ম্যাজিক স্ট্রিং, পুনরাবৃত্ত লিটারেল, হার্ডকোডেড IP, URL এবং কনফিগারেশন ফাইলের বাইরে এনভায়রনমেন্ট ভেরিয়েবল অ্যাক্সেস ধরুন। AigisCode সেই হার্ডওয়্যার্ড ভ্যালু খুঁজে যা রক্ষণাবেক্ষণ দুঃস্বপ্ন তৈরি করে।',
      fr: 'Attrapez les chaînes magiques, les littéraux répétés, les IPs en dur et les accès aux variables d\'environnement en dehors des fichiers de configuration.',
      es: 'Capture cadenas mágicas, literales repetidos, IPs hardcodeados y acceso a variables de entorno fuera de archivos de configuración.',
      cs: 'Zachyťte magické řetězce, opakované literály, natvrdo zadané IP adresy a přístup k proměnným prostředí mimo konfigurační soubory.',
    },
    metaDescription: {
      en: 'Detect magic strings, repeated literals, hardcoded IPs and URLs in your codebase. AigisCode finds hardwired values that should be configuration or constants.',
      zh: '检测代码库中的魔法字符串、重复字面量、硬编码IP和URL。',
      hi: 'अपने कोडबेस में मैजिक स्ट्रिंग्स, रिपीटेड लिटरल्स, हार्डकोडेड IPs और URLs का पता लगाएं।',
      pt: 'Detecte strings mágicas, literais repetidos, IPs e URLs hardcoded em sua base de código.',
      ar: 'اكتشف السلاسل السحرية والقيم الحرفية المتكررة وعناوين IP وURLs الثابتة في قاعدة شيفرتك. يجد AigisCode القيم الثابتة التي يجب أن تكون تكويناً أو ثوابت.',
      pl: 'Wykrywaj magiczne stringi, powtarzajace sie literaly, zakodowane adresy IP i URL-e w Twojej bazie kodu. AigisCode znajduje zakodowane wartosci, ktore powinny byc przeniesione do plikow konfiguracyjnych.',
      bn: 'আপনার কোডবেসে ম্যাজিক স্ট্রিং, পুনরাবৃত্ত লিটারেল, হার্ডকোডেড IP এবং URL শনাক্ত করুন। AigisCode সেই হার্ডওয়্যার্ড ভ্যালু খুঁজে যা কনফিগারেশন বা কনস্ট্যান্ট হওয়া উচিত।',
      fr: 'Détectez les chaînes magiques, littéraux répétés, IPs et URLs en dur dans votre code.',
      es: 'Detecte cadenas mágicas, literales repetidos, IPs y URLs hardcodeados en su base de código.',
      cs: 'Detekujte magické řetězce, opakované literály, natvrdo zadané IP a URL ve vaší kódové bázi.',
    },
    content: {
      en: `<h2 id="why-hardwired-values-matter">Why Hardwired Values Matter</h2>
<p>Hardwired values — magic strings, repeated literals, hardcoded URLs, and scattered environment variable access — are among the most common sources of bugs in production systems. When a database connection string appears in four different files and you need to change it, the odds of missing one instance are high. When an API endpoint URL is duplicated across services, updating it becomes a scavenger hunt through the codebase.</p>
<p>The problem is compounded in teams. Developer A hardcodes a timeout value of <code>30000</code> in one file, Developer B uses <code>30</code> (seconds instead of milliseconds) in another. Both think they are using the same timeout. Neither realizes the inconsistency until it causes a production incident.</p>

<h2 id="detection-categories">What AigisCode Detects</h2>
<p>The hardwiring detector identifies four categories of problematic values:</p>
<ul>
<li><strong>Magic strings</strong> — String literals that appear to be configuration values, API keys, file paths, or identifiers rather than UI text. AigisCode uses heuristics to distinguish translatable strings from configuration values.</li>
<li><strong>Repeated literals</strong> — The same string or numeric literal appearing in multiple files. By default, a literal must appear in at least 3 files to be flagged, but you can adjust this threshold via policy.</li>
<li><strong>Hardcoded network values</strong> — IP addresses, URLs, port numbers, and hostnames embedded directly in source code instead of being read from configuration.</li>
<li><strong>Environment access outside config</strong> — Calls to <code>process.env</code>, <code>os.environ</code>, <code>getenv()</code>, or <code>$_ENV</code> that occur outside designated configuration files. Scattering env access across the codebase makes it impossible to know what environment variables a project requires.</li>
</ul>

<h2 id="how-it-works">How the Detection Works</h2>
<p>During the Detect stage, AigisCode's hardwiring analyzer walks every parsed AST looking for string and numeric literals. Each literal is evaluated against a set of heuristics:</p>
<ol>
<li><strong>Pattern matching</strong> — Regex patterns detect IP addresses (<code>\\d+\\.\\d+\\.\\d+\\.\\d+</code>), URLs (<code>https?://</code>), email-like patterns, and file path patterns.</li>
<li><strong>Frequency analysis</strong> — Literals that appear across multiple files are flagged as repeated. The threshold is configurable via <code>repeated_literal_min_occurrences</code> in your policy.</li>
<li><strong>Context analysis</strong> — The detector considers where the literal appears. A string in a test assertion is less concerning than the same string in production middleware.</li>
<li><strong>Env access tracking</strong> — The analyzer identifies all calls to environment variable APIs and checks whether they occur in files matching your config path patterns.</li>
</ol>

<h2 id="practical-example">Practical Example</h2>
<p>In a Node.js application, AigisCode might report:</p>
<pre><code>Hardwiring Results:
├── 8 magic strings (API endpoints, file paths)
├── 5 repeated literals ("application/json" in 6 files)
├── 3 hardcoded IPs (192.168.1.100 in 2 files, localhost:3000 in 4 files)
└── 12 env accesses outside config (process.env.* in route handlers)</code></pre>
<p>Each finding includes the file path, line number, the literal value, and a suggested fix — typically extracting the value into a constants file or environment configuration module.</p>

<h2 id="policy-tuning">Policy Tuning</h2>
<p>Every project has legitimate hardcoded values. Test files often contain fixture data. Console commands contain help text. AigisCode's policy system lets you skip specific paths and adjust thresholds:</p>
<pre><code>{
  "hardwiring": {
    "repeated_literal_min_occurrences": 4,
    "skip_path_patterns": [
      "tests/**/*",
      "app/Console/**/*",
      "**/*.test.ts"
    ]
  }
}</code></pre>
<p>This focuses the report on production code and reduces noise from test fixtures and CLI help strings.</p>

<h2 id="benefits">Why This Matters for Teams</h2>
<p>Centralizing configuration values makes your application easier to deploy across environments, easier to audit for security compliance, and dramatically easier to maintain. AigisCode provides the visibility to get there systematically rather than hoping developers remember to extract every value.</p>`,
      zh: `<h2 id="why-hardwired-values-matter">为什么硬编码值很重要</h2>
<p>硬编码值——魔法字符串、重复字面量、硬编码URL和分散的环境变量访问——是生产系统中最常见的Bug来源。AigisCode检测四类问题值：魔法字符串、重复字面量、硬编码网络值和配置文件外的环境变量访问。通过策略系统自定义阈值和跳过路径。</p>`,
      fr: `<h2 id="why-hardwired-values-matter">Pourquoi les valeurs en dur sont importantes</h2>
<p>Les valeurs en dur sont parmi les sources de bugs les plus courantes. AigisCode détecte les chaînes magiques, les littéraux répétés, les valeurs réseau en dur et les accès env hors configuration. Le système de politique permet d'ajuster les seuils et d'exclure des chemins spécifiques.</p>`,
      es: `<h2 id="why-hardwired-values-matter">Por qué importan los valores hardcodeados</h2>
<p>Los valores hardcodeados son una de las fuentes más comunes de bugs en producción. AigisCode detecta cadenas mágicas, literales repetidos, valores de red hardcodeados y accesos a variables de entorno fuera de la configuración. El sistema de políticas permite ajustar umbrales y excluir rutas específicas.</p>`,
      cs: `<h2 id="why-hardwired-values-matter">Proč záleží na natvrdo zadaných hodnotách</h2>
<p>Natvrdo zadané hodnoty patří mezi nejčastější zdroje chyb v produkčních systémech. AigisCode detekuje magické řetězce, opakované literály, natvrdo zadané síťové hodnoty a přístupy k proměnným prostředí mimo konfiguraci. Systém politik umožňuje upravit prahy a vyloučit specifické cesty.</p>`,
      hi: `<h2 id="why-hardwired-values-matter">हार्डवायर्ड वैल्यू क्यों मायने रखती हैं</h2>
<p>हार्डवायर्ड वैल्यू प्रोडक्शन बग के सबसे सामान्य स्रोतों में से एक हैं। AigisCode मैजिक स्ट्रिंग, रिपीटेड लिटरल, हार्डकोडेड नेटवर्क वैल्यू और कॉन्फ़िग के बाहर env एक्सेस डिटेक्ट करता है। पॉलिसी सिस्टम थ्रेशोल्ड एडजस्ट करने और विशिष्ट पाथ एक्सक्लूड करने की अनुमति देता है।</p>`,
      pt: `<h2 id="why-hardwired-values-matter">Por que valores hardcoded importam</h2>
<p>Valores hardcoded estao entre as fontes mais comuns de bugs em producao. O AigisCode detecta strings magicas, literais repetidos, valores de rede hardcoded e acessos a variaveis de ambiente fora da configuracao. O sistema de politicas permite ajustar limites e excluir caminhos especificos.</p>`,
      ar: `<h2 id="why-hardwired-values-matter">لماذا تهم القيم الثابتة</h2>
<p>عناوين URLs المشفّرة وعناوين IP والسلاسل السحرية المبعثرة عبر الملفات تخلق كوابيس صيانة. يكتشف AigisCode أربع فئات من القيم الثابتة: السلاسل السحرية والقيم الحرفية المتكررة وعناوين IPs/URLs المشفّرة ووصول متغيرات البيئة خارج ملفات التكوين.</p>`,
      pl: `<h2 id="hardwired-values">Zakodowane wartosci</h2>
<p>Detektor identyfikuje magiczne stringi, powtarzajace sie literaly, zakodowane adresy sieciowe i dostep do zmiennych srodowiskowych poza warstwa konfiguracyjna.</p>`,
    },
    features: [
      'Magic string detection',
      'Repeated literal tracking',
      'Hardcoded IP/URL finder',
      'Env access outside config',
      'Configurable thresholds',
      'Path-based exclusions',
    ],
    codeExample: `# Analyze for hardwired values
aigiscode analyze /path/to/project

# View hardwiring findings
cat .aigiscode/deterministic-analysis.json | jq '.hardwiring'

# Customize thresholds in policy
echo '{"hardwiring": {"repeated_literal_min_occurrences": 4}}' > .aigiscode/policy.json`,
    relatedSlugs: ['dead-code-detection', 'laravel-project-analysis', 'django-project-analysis'],
  },
  /* ---- 4. AI-Assisted Code Review ---- */
  {
    slug: 'ai-assisted-code-review',
    icon: 'Robot',
    category: 'Workflow',
    title: {
      en: 'AI-Assisted Code Review',
      zh: 'AI辅助代码审查',
      hi: 'AI-सहायित कोड रिव्यू',
      pt: 'Revisão de Código Assistida por IA',
      ar: 'مراجعة الشيفرة بمساعدة الذكاء الاصطناعي',
      pl: 'Przeglad kodu wspomagany AI',
      bn: 'AI-সহায়তা কোড রিভিউ',
      fr: 'Revue de Code Assistée par IA',
      es: 'Revisión de Código Asistida por IA',
      cs: 'AI Asistovaná Revize Kódu',
    },
    shortDescription: {
      en: 'AI classifies findings as true positives, false positives, or needs-context — reducing noise so you focus on real issues. Works with OpenAI and Anthropic backends.',
      zh: 'AI将发现分类为真阳性、假阳性或需要上下文——减少噪音，让您专注于真正的问题。',
      hi: 'AI फाइंडिंग्स को true positive, false positive या needs-context में वर्गीकृत करता है।',
      pt: 'IA classifica descobertas como verdadeiros positivos, falsos positivos ou necessita contexto.',
      ar: 'يصنّف الذكاء الاصطناعي النتائج إلى إيجابيات حقيقية أو إيجابيات كاذبة أو تحتاج سياقاً — مما يقلل الضوضاء لتركّز على المشكلات الفعلية. يعمل مع واجهات OpenAI وAnthropic.',
      pl: 'AI klasyfikuje znaleziska jako prawdziwe problemy, falszywe alarmy lub wymagajace kontekstu. Proponuje strukturalne reguly wykluczajace, ktore utrzymuja sie miedzy uruchomieniami. Obsluguje backendy OpenAI i Anthropic.',
      bn: 'AI ফলাফলকে ট্রু পজিটিভ, ফলস পজিটিভ বা প্রসঙ্গ-নির্ভর হিসেবে শ্রেণিবদ্ধ করে — শব্দ কমিয়ে আপনাকে প্রকৃত সমস্যায় মনোযোগ দিতে দেয়। OpenAI এবং Anthropic ব্যাকএন্ডের সাথে কাজ করে।',
      fr: "L'IA classe les résultats en vrais positifs, faux positifs ou nécessitant du contexte.",
      es: 'La IA clasifica hallazgos como verdaderos positivos, falsos positivos o necesita contexto.',
      cs: 'AI klasifikuje nálezy jako true positive, false positive nebo needs-context.',
    },
    metaDescription: {
      en: 'Use AI to classify static analysis findings and reduce false positives. AigisCode integrates OpenAI and Anthropic to review dead code, cycles, and hardwiring findings.',
      zh: '使用AI对静态分析发现进行分类并减少误报。',
      hi: 'स्टैटिक एनालिसिस फाइंडिंग्स को क्लासिफाई करने के लिए AI का उपयोग करें।',
      pt: 'Use IA para classificar descobertas de análise estática e reduzir falsos positivos.',
      ar: 'استخدم الذكاء الاصطناعي لتصنيف نتائج التحليل الثابت وتقليل الإيجابيات الكاذبة. يدمج AigisCode واجهات OpenAI وAnthropic لمراجعة الشيفرة الميتة والدورات ونتائج القيم الثابتة.',
      pl: 'Uzyj AI do klasyfikacji znalezisk analizy statycznej i redukcji falszywych alarmow. AigisCode integruje OpenAI i Anthropic do inteligentnego sortowania znalezisk z trwalymi regulami wykluczajacymi.',
      bn: 'স্ট্যাটিক অ্যানালিসিস ফলাফল শ্রেণিবদ্ধ করতে এবং ফলস পজিটিভ কমাতে AI ব্যবহার করুন। AigisCode ডেড কোড, চক্র এবং হার্ডওয়্যারিং ফলাফল পর্যালোচনা করতে OpenAI এবং Anthropic ইন্টিগ্রেট করে।',
      fr: "Utilisez l'IA pour classifier les résultats d'analyse statique et réduire les faux positifs.",
      es: 'Use IA para clasificar hallazgos de análisis estático y reducir falsos positivos.',
      cs: 'Použijte AI ke klasifikaci nálezů statické analýzy a snížení falešně pozitivních výsledků.',
    },
    content: {
      en: `<h2 id="the-false-positive-problem">The False Positive Problem</h2>
<p>Every static analysis tool faces the same challenge: the more issues it detects, the more false positives it generates. A dead code detector might flag a class that is actually instantiated through a dependency injection container. A hardwiring detector might flag a URL that is intentionally hardcoded as a fallback default. When reports are full of noise, developers stop reading them.</p>
<p>AigisCode addresses this with a dedicated AI Review stage — the fifth stage in its six-stage pipeline. After the deterministic detectors have produced their candidates, an AI model reviews each finding in context and classifies it as a true positive, false positive, or needs-context.</p>

<h2 id="how-ai-review-works">How AI Review Works</h2>
<p>The AI Review stage operates as a final-pass classifier, not a first-pass detector. This is an important distinction. The deterministic detectors (dead code, hardwiring, cycles) use graph analysis and AST parsing to generate candidates with high recall. The AI model then acts as a precision filter:</p>
<ol>
<li><strong>Context assembly</strong> — For each finding, AigisCode assembles the relevant code context: the flagged code, surrounding lines, the file's imports, and related files from the dependency graph.</li>
<li><strong>Classification prompt</strong> — The assembled context is sent to the AI backend with a structured prompt asking for a classification: <code>true_positive</code>, <code>false_positive</code>, or <code>needs_context</code>.</li>
<li><strong>Confidence annotation</strong> — The AI response includes a confidence score and a brief explanation of the reasoning. This explanation appears in the final report alongside each finding.</li>
<li><strong>Report integration</strong> — Findings are annotated with their AI classification. The report separates confirmed true positives from likely false positives, making triage dramatically faster.</li>
</ol>

<h2 id="supported-backends">Supported AI Backends</h2>
<p>AigisCode supports two AI backends:</p>
<ul>
<li><strong>OpenAI</strong> — Works with GPT-4o and newer models. Configure via <code>OPENAI_API_KEY</code> environment variable.</li>
<li><strong>Anthropic</strong> — Works with Claude models. Configure via <code>ANTHROPIC_API_KEY</code> environment variable.</li>
</ul>
<p>You can also skip AI review entirely for fully deterministic analysis by passing the <code>--skip-ai</code> flag. This is useful for CI pipelines where you want fast, reproducible results without API costs.</p>

<h2 id="real-world-impact">Real-World Impact</h2>
<p>In testing across multiple production codebases, AI review typically reclassifies 15-30% of findings. Most reclassifications are false positive identifications — cases where the deterministic detector correctly found unused code that is actually accessed through framework conventions the detector does not model. For example:</p>
<pre><code>Finding: Abandoned class "UserObserver" — 0 references
AI Review: false_positive (confidence: 0.92)
Reason: "UserObserver follows the Laravel Observer naming convention
and is likely registered in AppServiceProvider via $this->app->observe().
The registration is string-based and would not appear in the import graph."</code></pre>
<p>This kind of framework-aware reasoning is exactly what deterministic analysis cannot provide and what AI review excels at.</p>

<h2 id="deterministic-first">Why Deterministic First, AI Second</h2>
<p>AigisCode's architecture deliberately places AI review after deterministic detection rather than replacing it. There are three reasons:</p>
<ul>
<li><strong>Reproducibility</strong> — Deterministic detectors produce the same results every time. AI classifications can vary between runs. By keeping the deterministic stage separate, you always have a stable baseline.</li>
<li><strong>Cost efficiency</strong> — Running AI review on every line of code would be prohibitively expensive. By first narrowing the field with graph analysis, AigisCode only sends dozens of findings to the AI model instead of thousands of files.</li>
<li><strong>Explainability</strong> — Each finding has a clear provenance: which detector found it, what graph evidence supports it, and what the AI model thinks about it. This makes the report auditable.</li>
</ul>

<h2 id="skipping-ai">Running Without AI</h2>
<p>For teams that want purely deterministic results or need to run analysis without API access, the <code>--skip-ai</code> flag bypasses the AI Review stage entirely. All findings from the Detect and Rules stages are included in the report without AI classification. This mode is faster, free, and fully reproducible.</p>`,
      zh: `<h2 id="the-false-positive-problem">假阳性问题</h2>
<p>每个静态分析工具都面临同样的挑战：检测到的问题越多，产生的假阳性就越多。AigisCode通过专门的AI审查阶段解决这个问题，在确定性检测器生成候选后，AI模型审查每个发现并分类为真阳性、假阳性或需要上下文。支持OpenAI和Anthropic后端。</p>`,
      fr: `<h2 id="the-false-positive-problem">Le problème des faux positifs</h2>
<p>AigisCode résout le problème des faux positifs avec une étape dédiée de revue IA. Après que les détecteurs déterministes ont produit leurs candidats, un modèle IA examine chaque résultat en contexte. Supporte les backends OpenAI et Anthropic. Utilisez <code>--skip-ai</code> pour une analyse purement déterministe.</p>`,
      es: `<h2 id="the-false-positive-problem">El problema de los falsos positivos</h2>
<p>AigisCode resuelve el problema de falsos positivos con una etapa dedicada de revisión IA. Después de que los detectores determinísticos producen candidatos, un modelo IA examina cada hallazgo en contexto. Soporta backends OpenAI y Anthropic. Use <code>--skip-ai</code> para análisis puramente determinístico.</p>`,
      cs: `<h2 id="the-false-positive-problem">Problém falešně pozitivních výsledků</h2>
<p>AigisCode řeší problém falešně pozitivních výsledků dedikovanou fází AI revize. Po tom, co deterministické detektory vytvoří kandidáty, AI model přezkoumá každý nález v kontextu. Podporuje backendy OpenAI a Anthropic. Použijte <code>--skip-ai</code> pro čistě deterministickou analýzu.</p>`,
      hi: `<h2 id="the-false-positive-problem">फ़ॉल्स पॉज़िटिव समस्या</h2>
<p>AigisCode एक डेडिकेटेड AI रिव्यू स्टेज से फ़ॉल्स पॉज़िटिव समस्या हल करता है। डिटर्मिनिस्टिक डिटेक्टर द्वारा कैंडिडेट बनाने के बाद, AI मॉडल कॉन्टेक्स्ट में हर फाइंडिंग की जांच करता है। OpenAI और Anthropic बैकएंड सपोर्ट करता है। पूर्णतया डिटर्मिनिस्टिक एनालिसिस के लिए <code>--skip-ai</code> का उपयोग करें।</p>`,
      pt: `<h2 id="the-false-positive-problem">O problema dos falsos positivos</h2>
<p>O AigisCode resolve o problema de falsos positivos com uma etapa dedicada de revisao IA. Apos os detectores deterministicos produzirem candidatos, um modelo IA examina cada descoberta em contexto. Suporta backends OpenAI e Anthropic. Use <code>--skip-ai</code> para analise puramente deterministica.</p>`,
      ar: `<h2 id="false-positive-problem">مشكلة الإيجابيات الكاذبة</h2>
<p>ينتج التحليل الثابت نتائج بعضها إيجابيات كاذبة من اتفاقيات الإطار. يستخدم AigisCode نماذج OpenAI وAnthropic لتصنيف النتائج كإيجابيات حقيقية أو كاذبة أو تحتاج سياقاً. يقترح قواعد استبعاد هيكلية لتقليل الضوضاء في التشغيلات المستقبلية.</p>`,
      pl: `<h2 id="ai-review">Przeglad kodu AI</h2>
<p>AI klasyfikuje znaleziska. Proponuje reguly wykluczajace zapisywane dla przyszlych uruchomien. Flaga <code>--skip-ai</code> dla deterministycznych wynikow w CI.</p>`,
    },
    features: [
      'True/false positive classification',
      'OpenAI backend support',
      'Anthropic backend support',
      'Context-aware reasoning',
      'Confidence scoring',
      'Skip-AI deterministic mode',
    ],
    codeExample: `# Full analysis with AI review
aigiscode analyze /path/to/project

# Deterministic only (no AI, no API costs)
aigiscode analyze /path/to/project

# Set AI backend via environment
export OPENAI_API_KEY=sk-...
aigiscode analyze /path/to/project`,
    relatedSlugs: ['circular-dependency-detection', 'dead-code-detection', 'ci-cd-integration'],
  },
  /* ---- 5. Laravel Project Analysis ---- */
  {
    slug: 'laravel-project-analysis',
    icon: 'Package',
    category: 'Framework',
    title: {
      en: 'Laravel Project Analysis',
      zh: 'Laravel项目分析',
      hi: 'Laravel प्रोजेक्ट एनालिसिस',
      pt: 'Análise de Projetos Laravel',
      ar: 'تحليل مشاريع Laravel',
      pl: 'Analiza projektow Laravel',
      bn: 'Laravel প্রজেক্ট বিশ্লেষণ',
      fr: 'Analyse de Projets Laravel',
      es: 'Análisis de Proyectos Laravel',
      cs: 'Analýza Laravel Projektů',
    },
    shortDescription: {
      en: 'Analyze Laravel codebases with built-in understanding of service providers, Eloquent models, middleware, and Blade templates. The Laravel plugin adapts detection to framework conventions.',
      zh: '内置对服务提供者、Eloquent模型、中间件和Blade模板的理解来分析Laravel代码库。',
      hi: 'सर्विस प्रोवाइडर्स, Eloquent मॉडल्स और Blade टेम्पलेट्स की समझ के साथ Laravel कोडबेस का विश्लेषण करें।',
      pt: 'Analise codebases Laravel com entendimento nativo de service providers, Eloquent, middleware e Blade.',
      ar: 'تحليل قواعد شيفرة Laravel مع فهم مدمج لمزوّدي الخدمات ونماذج Eloquent والوسيط وقوالب Blade. تكيّف إضافة Laravel الكشف مع اتفاقيات الإطار.',
      pl: 'Analizuj bazy kodu Laravel z wbudowanym rozumieniem service providerow, modeli Eloquent, lancuchow middleware, szablonow Blade i wzorow rejestracji routow.',
      bn: 'সার্ভিস প্রোভাইডার, Eloquent মডেল, মিডলওয়্যার এবং Blade টেমপ্লেটের বিল্ট-ইন বোঝাপড়া সহ Laravel কোডবেস বিশ্লেষণ করুন। Laravel plugin ফ্রেমওয়ার্ক কনভেনশনে ডিটেকশন মানিয়ে নেয়।',
      fr: 'Analysez les projets Laravel avec une compréhension native des service providers, Eloquent et Blade.',
      es: 'Analice proyectos Laravel con comprensión nativa de service providers, Eloquent, middleware y Blade.',
      cs: 'Analyzujte Laravel projekty s nativním porozuměním service providers, Eloquent a Blade.',
    },
    metaDescription: {
      en: 'Static analysis for Laravel projects. AigisCode understands service providers, Eloquent models, middleware chains, and Blade templates to find dead code and architectural issues.',
      zh: 'Laravel项目的静态分析。AigisCode理解服务提供者、Eloquent模型和Blade模板。',
      hi: 'Laravel प्रोजेक्ट्स के लिए स्टैटिक एनालिसिस।',
      pt: 'Análise estática para projetos Laravel com suporte nativo a service providers e Eloquent.',
      ar: 'تحليل ثابت لمشاريع Laravel. يفهم AigisCode مزوّدي الخدمات ونماذج Eloquent وسلاسل الوسيط وقوالب Blade للعثور على الشيفرة الميتة والمشكلات المعمارية.',
      pl: 'Analiza statyczna dla projektow Laravel. AigisCode rozumie service providery, modele Eloquent, lancuchy middleware, widoki Blade i rejestracje routow.',
      bn: 'Laravel প্রজেক্টের জন্য স্ট্যাটিক অ্যানালিসিস। AigisCode ডেড কোড এবং আর্কিটেকচারাল সমস্যা খুঁজতে সার্ভিস প্রোভাইডার, Eloquent মডেল, মিডলওয়্যার চেইন এবং Blade টেমপ্লেট বোঝে।',
      fr: 'Analyse statique pour projets Laravel avec support natif des service providers et Eloquent.',
      es: 'Análisis estático para proyectos Laravel con soporte nativo de service providers y Eloquent.',
      cs: 'Statická analýza pro Laravel projekty s nativní podporou service providers a Eloquent.',
    },
    content: {
      en: `<h2 id="laravel-challenges">Why Laravel Needs Specialized Analysis</h2>
<p>Laravel is one of the most popular PHP frameworks, powering millions of applications. But its reliance on service containers, string-based class resolution, implicit model binding, and convention-over-configuration patterns creates challenges for traditional static analysis tools. A standard dead code detector will flag Eloquent observers, event listeners, and policy classes as abandoned because they are registered through service providers using string references, not explicit imports.</p>
<p>AigisCode ships with a built-in Laravel plugin profile that understands these conventions and adjusts detection behavior accordingly.</p>

<h2 id="what-the-plugin-handles">What the Laravel Plugin Handles</h2>
<p>The Laravel plugin adapts AigisCode's detectors in several ways:</p>
<ul>
<li><strong>Service provider awareness</strong> — Classes registered via <code>$this->app->bind()</code>, <code>$this->app->singleton()</code>, and <code>$this->app->register()</code> are not flagged as abandoned even if no file directly imports them.</li>
<li><strong>Eloquent model relationships</strong> — String-based references in <code>belongsTo()</code>, <code>hasMany()</code>, <code>morphTo()</code> and other relationship methods are traced as dependencies.</li>
<li><strong>Middleware chains</strong> — Middleware classes referenced in <code>app/Http/Kernel.php</code> or route group definitions are recognized as entry points.</li>
<li><strong>Event/listener mapping</strong> — Classes in the <code>$listen</code> array of <code>EventServiceProvider</code> are recognized as referenced.</li>
<li><strong>Config and env patterns</strong> — The plugin knows that <code>config/</code> files are the canonical location for <code>env()</code> calls. Access to <code>env()</code> outside <code>config/</code> is flagged as hardwiring.</li>
<li><strong>Artisan commands</strong> — Console commands in <code>app/Console/Commands/</code> are treated as entry points.</li>
</ul>

<h2 id="common-findings">Common Findings in Laravel Projects</h2>
<p>Even with the Laravel plugin active, AigisCode typically finds significant issues in mature Laravel projects:</p>
<pre><code>$ aigiscode analyze /path/to/laravel-app
  Results:
  ├── 2 circular dependencies (Controller → Service → Repository → Controller)
  ├── 15 unused imports
  ├── 4 abandoned classes (old feature controllers never removed)
  ├── 8 env() calls outside config/ directory
  ├── 3 repeated literals ("api/v2" in 5 files)
  └── 2 god classes (UserController: 45 methods, OrderService: 38 methods)</code></pre>
<p>The circular dependency between controllers, services, and repositories is a classic Laravel anti-pattern that grows as applications scale. AigisCode identifies the exact cycle so you can refactor with confidence.</p>

<h2 id="god-classes">Detecting God Classes</h2>
<p>Laravel applications often develop god classes — single controllers or services that accumulate dozens of methods over time. AigisCode's graph analyzer flags classes that exceed responsibility thresholds based on method count, incoming dependency count, and lines of code. Breaking these into smaller, focused classes improves testability and reduces merge conflicts.</p>

<h2 id="setup">Setting Up for Laravel</h2>
<p>AigisCode auto-detects Laravel projects by looking for <code>artisan</code>, <code>composer.json</code> with <code>laravel/framework</code>, and the standard directory structure. The Laravel plugin is activated automatically. You can customize it further with a policy file:</p>
<pre><code>{
  "graph": {
    "orphan_entry_patterns": [
      "app/Console/Commands/**/*.php",
      "app/Providers/**/*.php",
      "database/migrations/**/*.php",
      "database/seeders/**/*.php"
    ]
  },
  "hardwiring": {
    "skip_path_patterns": ["config/**/*.php"]
  }
}</code></pre>
<p>This ensures that Artisan commands, service providers, migrations, and seeders are never flagged as orphan files, and config files are not scanned for hardwired values since they are the canonical config location.</p>

<h2 id="workflow">Recommended Workflow</h2>
<p>For Laravel teams, we recommend running AigisCode as part of your CI pipeline on every pull request. Start by generating a baseline report, then set up rules to exclude known false positives. Over time, the exclusion rules stabilize and the tool surfaces only new issues. This catch-as-you-go approach prevents architectural debt from accumulating.</p>`,
      zh: `<h2 id="laravel-challenges">为什么Laravel需要专门的分析</h2>
<p>Laravel依赖服务容器、基于字符串的类解析和约定优于配置的模式，这给传统静态分析工具带来挑战。AigisCode内置Laravel插件配置文件，理解服务提供者、Eloquent模型关系、中间件链、事件/监听器映射和Artisan命令。插件自动激活，可通过策略文件进一步自定义。</p>`,
      fr: `<h2 id="laravel-challenges">Pourquoi Laravel nécessite une analyse spécialisée</h2>
<p>Le plugin Laravel d'AigisCode comprend les service providers, les relations Eloquent, les chaînes de middleware et le mapping événements/listeners. Il est activé automatiquement et ajuste les détecteurs pour réduire les faux positifs liés aux conventions du framework.</p>`,
      es: `<h2 id="laravel-challenges">Por qué Laravel necesita análisis especializado</h2>
<p>El plugin Laravel de AigisCode comprende service providers, relaciones Eloquent, cadenas de middleware y mapeo de eventos/listeners. Se activa automáticamente y ajusta los detectores para reducir falsos positivos relacionados con las convenciones del framework.</p>`,
      cs: `<h2 id="laravel-challenges">Proč Laravel potřebuje specializovanou analýzu</h2>
<p>Laravel plugin AigisCode rozumí service providers, Eloquent relacím, middleware řetězcům a mapování událostí/listenerů. Automaticky se aktivuje a upravuje detektory pro snížení falešně pozitivních výsledků spojených s konvencemi frameworku.</p>`,
      hi: `<h2 id="laravel-challenges">Laravel को विशेष विश्लेषण की आवश्यकता क्यों है</h2>
<p>AigisCode का Laravel प्लगइन सर्विस प्रोवाइडर, Eloquent रिलेशनशिप, मिडलवेयर चेन और इवेंट/लिसनर मैपिंग समझता है। यह ऑटोमैटिकली एक्टिवेट होता है और फ्रेमवर्क कन्वेंशन से जुड़े फ़ॉल्स पॉज़िटिव कम करने के लिए डिटेक्टर एडजस्ट करता है।</p>`,
      pt: `<h2 id="laravel-challenges">Por que Laravel precisa de analise especializada</h2>
<p>O plugin Laravel do AigisCode compreende service providers, relacoes Eloquent, cadeias de middleware e mapeamento de eventos/listeners. E ativado automaticamente e ajusta os detectores para reduzir falsos positivos relacionados as convencoes do framework.</p>`,
      ar: `<h2 id="why-laravel-needs-analysis">لماذا يحتاج Laravel تحليلاً متخصصاً</h2>
<p>يعتمد Laravel بكثرة على مزوّدي الخدمات وتسجيل الحاوية وواجهات Facade. كثير من الشيفرة "غير المرجعية" ظاهرياً نشطة عبر حاوية الخدمات. تفهم إضافة AigisCode لـ Laravel مزوّدي الخدمات ونماذج Eloquent وسلاسل الوسيط وقوالب Blade لتقديم نتائج دقيقة.</p>`,
      pl: `<h2 id="laravel">Analiza Laravel</h2>
<p>Plugin rozumie service providery, Eloquent, middleware, Blade i routy. Automatyczne wykrywanie i polityki frameworka.</p>`,
    },
    features: [
      'Auto-detected Laravel plugin',
      'Service provider awareness',
      'Eloquent relationship tracing',
      'Middleware chain recognition',
      'God class detection',
      'Env-outside-config detection',
    ],
    codeExample: `# Analyze a Laravel project (plugin auto-activates)
aigiscode analyze /path/to/laravel-app

# View god classes
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.god_classes'

# View circular dependencies
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.strong_circular_dependencies'`,
    relatedSlugs: ['wordpress-analysis', 'hardwired-values-detection', 'circular-dependency-detection'],
  },
  /* ---- 6. Django Project Analysis ---- */
  {
    slug: 'django-project-analysis',
    icon: 'Flask',
    category: 'Framework',
    title: {
      en: 'Django Project Analysis',
      zh: 'Django项目分析',
      hi: 'Django प्रोजेक्ट एनालिसिस',
      pt: 'Análise de Projetos Django',
      ar: 'تحليل مشاريع Django',
      pl: 'Analiza projektow Django',
      bn: 'Django প্রজেক্ট বিশ্লেষণ',
      fr: 'Analyse de Projets Django',
      es: 'Análisis de Proyectos Django',
      cs: 'Analýza Django Projektů',
    },
    shortDescription: {
      en: 'Analyze Django applications with awareness of models, views, URL routing, management commands, and signal handlers. The Django plugin reduces false positives from framework conventions.',
      zh: '分析Django应用程序，感知模型、视图、URL路由、管理命令和信号处理程序。',
      hi: 'मॉडल्स, व्यूज, URL रूटिंग और सिग्नल हैंडलर्स की जागरूकता के साथ Django एप्लिकेशन का विश्लेषण करें।',
      pt: 'Analise aplicações Django com conhecimento de models, views, URL routing e signal handlers.',
      ar: 'تحليل تطبيقات Django مع الوعي بالنماذج والعروض وتوجيه عناوين URL وأوامر الإدارة ومعالجات الإشارات. تقلل إضافة Django الإيجابيات الكاذبة من اتفاقيات الإطار.',
      pl: 'Analizuj aplikacje Django ze swiadomoscia modeli, widokow, routingu URL, polecen zarzadzania i obslugi sygnalow. Natywna dla Pythona konstrukcja grafu zaleznosci.',
      bn: 'মডেল, ভিউ, URL রাউটিং, ম্যানেজমেন্ট কমান্ড এবং সিগনাল হ্যান্ডলারের সচেতনতা সহ Django অ্যাপ্লিকেশন বিশ্লেষণ করুন। Django plugin ফ্রেমওয়ার্ক কনভেনশন থেকে ফলস পজিটিভ কমায়।',
      fr: 'Analysez les applications Django avec connaissance des models, views, URL routing et signal handlers.',
      es: 'Analice aplicaciones Django con conocimiento de models, views, URL routing y signal handlers.',
      cs: 'Analyzujte Django aplikace s povědomím o modelech, views, URL routing a signal handlers.',
    },
    metaDescription: {
      en: 'Static analysis tailored for Django projects. AigisCode understands Django models, views, URL patterns, signals, and management commands to deliver accurate findings.',
      zh: '为Django项目量身定制的静态分析。AigisCode理解Django模型、视图和URL模式。',
      hi: 'Django प्रोजेक्ट्स के लिए अनुकूलित स्टैटिक एनालिसिस।',
      pt: 'Análise estática adaptada para projetos Django.',
      ar: 'تحليل ثابت مخصص لمشاريع Django. يفهم AigisCode نماذج Django والعروض وأنماط URL والإشارات وأوامر الإدارة لتقديم نتائج دقيقة.',
      pl: 'Analiza statyczna dostosowana do projektow Django. AigisCode rozumie modele Django, widoki, wzorce URL, polecenia zarzadzania i obsluge sygnalow.',
      bn: 'Django প্রজেক্টের জন্য বিশেষভাবে তৈরি স্ট্যাটিক অ্যানালিসিস। AigisCode সঠিক ফলাফল দিতে Django মডেল, ভিউ, URL প্যাটার্ন, সিগনাল এবং ম্যানেজমেন্ট কমান্ড বোঝে।',
      fr: 'Analyse statique adaptée pour les projets Django.',
      es: 'Análisis estático adaptado para proyectos Django.',
      cs: 'Statická analýza přizpůsobená pro Django projekty.',
    },
    content: {
      en: `<h2 id="django-conventions">Django's Convention-Heavy Architecture</h2>
<p>Django follows a convention-over-configuration philosophy that makes development fast but creates challenges for static analysis. URL patterns reference views by string path or function reference. Models are auto-discovered from <code>models.py</code> in each app. Signal handlers are connected in <code>apps.py</code> or <code>signals.py</code> using decorator-based registration. Management commands are discovered by directory convention. Template tags are loaded by name, not by import.</p>
<p>Without framework awareness, a static analysis tool would flag half of a Django project as dead code — management commands, signal handlers, template tags, and admin registrations all lack explicit import chains.</p>

<h2 id="django-plugin">What the Django Plugin Does</h2>
<p>AigisCode's built-in Django plugin adjusts detection to account for framework conventions:</p>
<ul>
<li><strong>URL routing</strong> — Views referenced in <code>urls.py</code> through <code>path()</code>, <code>re_path()</code>, and <code>include()</code> are traced as dependencies, even when referenced by string.</li>
<li><strong>Model auto-discovery</strong> — Classes in <code>models.py</code> that inherit from <code>django.db.models.Model</code> are recognized as entry points since Django discovers them automatically.</li>
<li><strong>Signal handlers</strong> — Functions decorated with <code>@receiver</code> or connected via <code>signal.connect()</code> in <code>apps.py</code> are not flagged as unreferenced.</li>
<li><strong>Management commands</strong> — Classes in <code>management/commands/</code> inheriting from <code>BaseCommand</code> are recognized as entry points.</li>
<li><strong>Admin registrations</strong> — Classes registered via <code>admin.site.register()</code> or <code>@admin.register()</code> decorators are linked to their models.</li>
<li><strong>Settings module</strong> — The <code>settings.py</code> file (or settings package) is recognized as the canonical location for configuration values. Hardcoded values in settings are not flagged.</li>
</ul>

<h2 id="common-findings">Common Findings in Django Projects</h2>
<p>Django projects that have been in development for more than a year typically accumulate these issues:</p>
<pre><code>$ aigiscode analyze /path/to/django-project
  Results:
  ├── 3 circular dependencies between apps
  ├── 18 unused imports across views and serializers
  ├── 6 abandoned model methods (never called externally)
  ├── 4 orphan utility files from deprecated features
  ├── 7 hardcoded URLs in views (should use reverse())
  └── 2 god classes (UserViewSet: 32 methods)</code></pre>
<p>The circular dependencies between Django apps are particularly problematic because they prevent you from extracting apps into separate packages. AigisCode identifies the exact import chains that create the coupling.</p>

<h2 id="cross-app-dependencies">Cross-App Dependency Analysis</h2>
<p>One of AigisCode's most valuable features for Django projects is cross-app dependency analysis. Django encourages organizing code into reusable apps, but in practice, apps often develop tight coupling. AigisCode's dependency graph shows you which apps depend on which, where the coupling is strongest, and which apps could be extracted as standalone packages. This visibility is essential for teams planning to modularize their Django monolith.</p>

<h2 id="migration-analysis">Migration File Handling</h2>
<p>Migration files are auto-generated by Django and should not be flagged as dead code or scanned for hardwired values. The Django plugin automatically excludes <code>migrations/</code> directories from dead code detection and hardwiring scans. Custom migration operations are still analyzed for circular dependencies.</p>

<h2 id="setup">Getting Started with Django</h2>
<p>AigisCode auto-detects Django projects by looking for <code>manage.py</code>, <code>settings.py</code>, and the standard app structure. The Django plugin activates automatically. Run the analysis and check the report:</p>
<pre><code>cd /path/to/django-project
aigiscode analyze .
cat .aigiscode/deterministic-analysis.json | jq '.summary'</code></pre>`,
      zh: `<h2 id="django-conventions">Django的约定驱动架构</h2>
<p>Django遵循约定优于配置的理念。AigisCode的Django插件理解URL路由、模型自动发现、信号处理程序、管理命令和Admin注册。插件自动激活，减少因框架约定产生的假阳性。常见发现包括应用间循环依赖、未使用的导入和硬编码URL。</p>`,
      fr: `<h2 id="django-conventions">L'architecture conventionnelle de Django</h2>
<p>Le plugin Django d'AigisCode comprend le routage URL, l'auto-découverte des modèles, les signal handlers, les commandes de gestion et les enregistrements admin. Il est activé automatiquement pour réduire les faux positifs liés aux conventions Django.</p>`,
      es: `<h2 id="django-conventions">La arquitectura basada en convenciones de Django</h2>
<p>El plugin Django de AigisCode comprende el enrutamiento URL, el auto-descubrimiento de modelos, signal handlers, comandos de gestión y registros admin. Se activa automáticamente para reducir falsos positivos relacionados con las convenciones de Django.</p>`,
      cs: `<h2 id="django-conventions">Konvenční architektura Djanga</h2>
<p>Django plugin AigisCode rozumí URL routingu, auto-discovery modelů, signal handlerům, management příkazům a admin registracím. Automaticky se aktivuje pro snížení falešně pozitivních výsledků spojených s Django konvencemi.</p>`,
      hi: `<h2 id="django-conventions">Django की कन्वेंशन-ड्रिवन आर्किटेक्चर</h2>
<p>AigisCode का Django प्लगइन URL रूटिंग, मॉडल ऑटो-डिस्कवरी, सिग्नल हैंडलर, मैनेजमेंट कमांड और admin रजिस्ट्रेशन समझता है। Django कन्वेंशन से जुड़े फ़ॉल्स पॉज़िटिव कम करने के लिए ऑटोमैटिकली एक्टिवेट होता है।</p>`,
      pt: `<h2 id="django-conventions">A arquitetura baseada em convencoes do Django</h2>
<p>O plugin Django do AigisCode compreende roteamento de URLs, auto-descoberta de modelos, signal handlers, comandos de gerenciamento e registros admin. E ativado automaticamente para reduzir falsos positivos relacionados as convencoes do Django.</p>`,
      ar: `<h2 id="django-conventions">البنية الغنية بالاتفاقيات في Django</h2>
<p>Django يعتمد على الاتفاقيات: النماذج والعروض وأنماط URL والإشارات وأوامر الإدارة كلها تُكتشف تلقائياً. تفهم إضافة AigisCode لـ Django هذه الاتفاقيات وتمنع الإيجابيات الكاذبة مع اكتشاف المشكلات المعمارية الحقيقية.</p>`,
      pl: `<h2 id="django">Analiza Django</h2>
<p>Swiadomosc modeli, widokow, URL, polecen zarzadzania i sygnalow. Natywne parsowanie tree-sitter-python.</p>`,
    },
    features: [
      'Auto-detected Django plugin',
      'URL routing analysis',
      'Model auto-discovery',
      'Signal handler recognition',
      'Cross-app dependency mapping',
      'Migration file exclusion',
    ],
    codeExample: `# Analyze a Django project (plugin auto-activates)
aigiscode analyze /path/to/django-project

# View cross-app dependencies
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis'

# Check for hardcoded URLs in views
cat .aigiscode/deterministic-analysis.json | jq '.hardwiring'`,
    relatedSlugs: ['laravel-project-analysis', 'python-codebase-analysis', 'circular-dependency-detection'],
  },
  /* ---- 7. WordPress Analysis ---- */
  {
    slug: 'wordpress-analysis',
    icon: 'Globe',
    category: 'Framework',
    title: {
      en: 'WordPress Theme & Plugin Analysis',
      zh: 'WordPress主题和插件分析',
      hi: 'WordPress थीम और प्लगइन एनालिसिस',
      pt: 'Análise de Temas e Plugins WordPress',
      ar: 'تحليل قوالب وإضافات WordPress',
      pl: 'Analiza szablonow i pluginow WordPress',
      bn: 'WordPress থিম ও Plugin বিশ্লেষণ',
      fr: 'Analyse de Thèmes et Plugins WordPress',
      es: 'Análisis de Temas y Plugins WordPress',
      cs: 'Analýza WordPress Témat a Pluginů',
    },
    shortDescription: {
      en: 'Analyze WordPress themes and plugins with understanding of hooks, filters, shortcodes, and the WordPress plugin API. Find dead code and hardwired values in PHP-based WordPress projects.',
      zh: '分析WordPress主题和插件，理解钩子、过滤器、短代码和WordPress插件API。',
      hi: 'हुक्स, फिल्टर्स और शॉर्टकोड्स की समझ के साथ WordPress थीम और प्लगइन्स का विश्लेषण करें।',
      pt: 'Analise temas e plugins WordPress com entendimento de hooks, filtros e shortcodes.',
      ar: 'تحليل قوالب وإضافات WordPress مع فهم الخطافات والمرشحات والاختصارات وواجهة WordPress البرمجية للإضافات. اعثر على الشيفرة الميتة والقيم الثابتة في مشاريع WordPress المبنية بـ PHP.',
      pl: 'Analizuj szablony i pluginy WordPress z rozumieniem hookow, filtrow, shortcodow i architektury pluginow WordPress. Rozpoznaje rejestracje add_action i add_filter jako prawidlowe punkty wejscia.',
      bn: 'হুক, ফিল্টার, শর্টকোড এবং WordPress plugin API-এর বোঝাপড়া সহ WordPress থিম এবং plugin বিশ্লেষণ করুন। PHP-ভিত্তিক WordPress প্রজেক্টে ডেড কোড এবং হার্ডওয়্যার্ড ভ্যালু খুঁজুন।',
      fr: 'Analysez les thèmes et plugins WordPress avec compréhension des hooks, filtres et shortcodes.',
      es: 'Analice temas y plugins WordPress con comprensión de hooks, filtros y shortcodes.',
      cs: 'Analyzujte WordPress témata a pluginy s porozuměním hookům, filtrům a shortcodes.',
    },
    metaDescription: {
      en: 'Static analysis for WordPress themes and plugins. AigisCode understands WordPress hooks, filters, actions, and shortcodes to find dead code and architectural issues in PHP.',
      zh: 'WordPress主题和插件的静态分析。',
      hi: 'WordPress थीम और प्लगइन्स के लिए स्टैटिक एनालिसिस।',
      pt: 'Análise estática para temas e plugins WordPress.',
      ar: 'تحليل ثابت لقوالب وإضافات WordPress. يفهم AigisCode خطافات ومرشحات وأحداث واختصارات WordPress للعثور على الشيفرة الميتة والمشكلات المعمارية في PHP.',
      pl: 'Analiza statyczna dla szablonow i pluginow WordPress. AigisCode rozumie hooki WordPress, filtry, akcje, shortcody i wzorce bootstrap pluginow.',
      bn: 'WordPress থিম এবং plugin-এর জন্য স্ট্যাটিক অ্যানালিসিস। AigisCode PHP-তে ডেড কোড এবং আর্কিটেকচারাল সমস্যা খুঁজতে WordPress হুক, ফিল্টার, অ্যাকশন এবং শর্টকোড বোঝে।',
      fr: 'Analyse statique pour les thèmes et plugins WordPress.',
      es: 'Análisis estático para temas y plugins WordPress.',
      cs: 'Statická analýza pro WordPress témata a pluginy.',
    },
    content: {
      en: `<h2 id="wordpress-ecosystem">The WordPress Analysis Challenge</h2>
<p>WordPress powers over 40% of the web, yet its PHP codebase is notoriously difficult to analyze statically. The hook system (<code>add_action</code>, <code>add_filter</code>) creates invisible connections between functions. Shortcodes are registered by string name. Theme template files are loaded by WordPress's template hierarchy, not by explicit PHP includes. Widget classes are registered through <code>register_widget()</code>. None of these relationships appear in a traditional import graph.</p>
<p>AigisCode's WordPress plugin profile understands these conventions and traces the implicit connections that make up a WordPress project's real dependency graph.</p>

<h2 id="hook-system">Understanding the Hook System</h2>
<p>WordPress's hook system is the backbone of its extensibility — and the biggest challenge for static analysis. When a plugin calls <code>add_action('init', 'my_custom_function')</code>, the function <code>my_custom_function</code> is called during WordPress initialization, but no file explicitly imports or calls it. AigisCode's WordPress plugin:</p>
<ul>
<li>Parses all <code>add_action()</code> and <code>add_filter()</code> calls to build a hook-to-function mapping</li>
<li>Traces <code>do_action()</code> and <code>apply_filters()</code> calls to determine which hooks are actually fired</li>
<li>Marks hooked functions as referenced, preventing false dead code reports</li>
<li>Reports orphan hooks — functions attached to hooks that are never triggered</li>
</ul>

<h2 id="common-findings">Common Findings in WordPress Projects</h2>
<p>WordPress themes and plugins that have been maintained over multiple versions typically accumulate these issues:</p>
<pre><code>$ aigiscode analyze /path/to/wordpress-plugin
  Results:
  ├── 12 unused imports/requires
  ├── 5 functions hooked to deprecated hooks
  ├── 8 hardcoded URLs (wp-admin paths, API endpoints)
  ├── 3 repeated literals (version strings in 6 files)
  ├── 4 orphan template files (never loaded by template hierarchy)
  └── 2 god classes (Main_Plugin_Class: 52 methods)</code></pre>
<p>Functions hooked to deprecated WordPress hooks are a particularly valuable finding — they represent code that once worked but is silently ignored in newer WordPress versions.</p>

<h2 id="template-hierarchy">Template Hierarchy Analysis</h2>
<p>WordPress loads template files through its template hierarchy — <code>single.php</code>, <code>archive.php</code>, <code>page-{slug}.php</code>, etc. AigisCode understands this hierarchy and only flags template files as orphans if they do not match any hierarchy pattern. Custom template files referenced via <code>get_template_part()</code> are also traced.</p>

<h2 id="security-patterns">Security-Relevant Findings</h2>
<p>WordPress plugins are frequent targets for security exploits. AigisCode helps identify patterns that suggest security risks:</p>
<ul>
<li><strong>Hardcoded API keys</strong> — Credentials embedded in plugin code instead of using <code>wp_options</code> or environment variables.</li>
<li><strong>Direct database queries</strong> — Calls to <code>$wpdb->query()</code> with hardcoded SQL instead of prepared statements.</li>
<li><strong>Hardcoded nonce names</strong> — Nonce strings duplicated across files instead of being centralized.</li>
</ul>

<h2 id="setup">Running the Analysis</h2>
<p>Point AigisCode at your WordPress plugin or theme directory. The WordPress plugin profile is auto-detected when the project contains <code>functions.php</code>, a <code>style.css</code> with a WordPress theme header, or a plugin header comment in the main PHP file:</p>
<pre><code>aigiscode analyze /path/to/wp-content/plugins/my-plugin
aigiscode analyze /path/to/wp-content/themes/my-theme</code></pre>
<p>The report is written to <code>.aigiscode/deterministic-analysis.json</code> inside the analyzed directory.</p>`,
      zh: `<h2 id="wordpress-ecosystem">WordPress分析挑战</h2>
<p>WordPress的钩子系统创建了函数之间不可见的连接。AigisCode的WordPress插件理解钩子系统、短代码注册、模板层次结构和小部件注册。它解析所有add_action和add_filter调用来构建钩子到函数的映射，防止死代码的错误报告。</p>`,
      fr: `<h2 id="wordpress-ecosystem">Le défi de l'analyse WordPress</h2>
<p>Le plugin WordPress d'AigisCode comprend le système de hooks, l'enregistrement des shortcodes, la hiérarchie des templates et l'enregistrement des widgets. Il analyse les appels add_action et add_filter pour construire un mapping hooks-fonctions et prévenir les faux rapports de code mort.</p>`,
      es: `<h2 id="wordpress-ecosystem">El desafío del análisis WordPress</h2>
<p>El plugin WordPress de AigisCode comprende el sistema de hooks, registro de shortcodes, jerarquía de templates y registro de widgets. Analiza las llamadas add_action y add_filter para construir un mapeo hooks-funciones y prevenir reportes falsos de código muerto.</p>`,
      cs: `<h2 id="wordpress-ecosystem">Výzva analýzy WordPressu</h2>
<p>WordPress plugin AigisCode rozumí systému hooků, registraci shortcodes, hierarchii šablon a registraci widgetů. Analyzuje volání add_action a add_filter pro vytvoření mapování hooků na funkce a prevenci falešných hlášení mrtvého kódu.</p>`,
      hi: `<h2 id="wordpress-ecosystem">WordPress विश्लेषण की चुनौती</h2>
<p>AigisCode का WordPress प्लगइन हुक सिस्टम, शॉर्टकोड रजिस्ट्रेशन, टेम्पलेट हायरार्की और विजेट रजिस्ट्रेशन समझता है। यह add_action और add_filter कॉल पार्स करके हुक-टू-फ़ंक्शन मैपिंग बनाता है और डेड कोड के फ़ॉल्स रिपोर्ट रोकता है।</p>`,
      pt: `<h2 id="wordpress-ecosystem">O desafio da analise WordPress</h2>
<p>O plugin WordPress do AigisCode compreende o sistema de hooks, registro de shortcodes, hierarquia de templates e registro de widgets. Analisa chamadas add_action e add_filter para construir um mapeamento hooks-funcoes e prevenir relatorios falsos de codigo morto.</p>`,
      ar: `<h2 id="wordpress-challenge">تحدي تحليل WordPress</h2>
<p>يعتمد WordPress على خطافات ومرشحات وأحداث وشيفرات قصيرة بدلاً من الاستيرادات المباشرة. كثير من الشيفرة "الميتة" ظاهرياً مُسجّلة عبر نظام الخطافات. تفهم إضافة AigisCode لـ WordPress واجهة الإضافات والخطافات وتحليل القوالب للعثور على المشكلات الحقيقية.</p>`,
      pl: `<h2 id="wordpress">Analiza WordPress</h2>
<p>Hooki, filtry, shortcody i architektura pluginow. Rejestracje <code>add_action</code> i <code>add_filter</code> jako punkty wejscia.</p>`,
    },
    features: [
      'Hook system analysis',
      'Filter chain tracing',
      'Template hierarchy awareness',
      'Shortcode registration tracking',
      'Deprecated hook detection',
      'Security pattern identification',
    ],
    codeExample: `# Analyze a WordPress plugin
aigiscode analyze /path/to/wp-content/plugins/my-plugin

# Analyze a WordPress theme
aigiscode analyze /path/to/wp-content/themes/my-theme

# Check for deprecated hooks
cat .aigiscode/deterministic-analysis.json | jq '.dead_code'`,
    relatedSlugs: ['laravel-project-analysis', 'hardwired-values-detection', 'dead-code-detection'],
  },
  /* ---- 8. TypeScript Monorepo Analysis ---- */
  {
    slug: 'typescript-monorepo-analysis',
    icon: 'TreeStructure',
    category: 'Workflow',
    title: {
      en: 'TypeScript Monorepo Analysis',
      zh: 'TypeScript Monorepo分析',
      hi: 'TypeScript Monorepo एनालिसिस',
      pt: 'Análise de Monorepo TypeScript',
      ar: 'تحليل مستودعات TypeScript الأحادية',
      pl: 'Analiza monorepo TypeScript',
      bn: 'TypeScript মনোরিপো বিশ্লেষণ',
      fr: 'Analyse de Monorepo TypeScript',
      es: 'Análisis de Monorepo TypeScript',
      cs: 'Analýza TypeScript Monorepo',
    },
    shortDescription: {
      en: 'Analyze large TypeScript monorepos with path alias resolution, cross-package dependency tracking, and barrel file analysis. Find architectural issues across hundreds of packages.',
      zh: '分析大型TypeScript monorepo，支持路径别名解析、跨包依赖追踪和barrel文件分析。',
      hi: 'पाथ एलियास रेजोल्यूशन और क्रॉस-पैकेज डिपेंडेंसी ट्रैकिंग के साथ बड़े TypeScript monorepos का विश्लेषण करें।',
      pt: 'Analise grandes monorepos TypeScript com resolução de path aliases e rastreamento de dependências entre pacotes.',
      ar: 'تحليل مستودعات TypeScript الأحادية الكبيرة مع حل أسماء المسارات المستعارة وتتبع التبعيات عبر الحزم وتحليل ملفات barrel. اعثر على المشكلات المعمارية عبر مئات الحزم.',
      pl: 'Analizuj duze monorepa TypeScript z rozwiazywaniem aliasow sciezek, sledzeniem zaleznosci miedzy pakietami i obsluga plikow deklaracji. Wykrywaj miedzy-pakietowe cykliczne zaleznosci.',
      bn: 'পাথ অ্যালিয়াস রেজোলিউশন, ক্রস-প্যাকেজ ডিপেন্ডেন্সি ট্র্যাকিং এবং ব্যারেল ফাইল বিশ্লেষণ সহ বড় TypeScript মনোরিপো বিশ্লেষণ করুন। শত শত প্যাকেজ জুড়ে আর্কিটেকচারাল সমস্যা খুঁজুন।',
      fr: 'Analysez de grands monorepos TypeScript avec résolution d\'alias de chemins et suivi des dépendances entre packages.',
      es: 'Analice grandes monorepos TypeScript con resolución de alias de rutas y seguimiento de dependencias entre paquetes.',
      cs: 'Analyzujte velké TypeScript monorepo s rozlišením aliasů cest a sledováním závislostí mezi balíčky.',
    },
    metaDescription: {
      en: 'Static analysis for TypeScript monorepos. AigisCode resolves path aliases, tracks cross-package dependencies, and finds circular imports across your entire monorepo.',
      zh: 'TypeScript monorepo的静态分析。',
      hi: 'TypeScript monorepos के लिए स्टैटिक एनालिसिस।',
      pt: 'Análise estática para monorepos TypeScript.',
      ar: 'تحليل ثابت لمستودعات TypeScript الأحادية. يحل AigisCode أسماء المسارات المستعارة ويتتبع التبعيات عبر الحزم ويجد الاستيرادات الدائرية عبر مستودعك الأحادي بالكامل.',
      pl: 'Analiza statyczna dla monorepo TypeScript. AigisCode rozwiazuje aliasy sciezek, sledzi zaleznosci miedzy pakietami i obsluguje pliki deklaracji.',
      bn: 'TypeScript মনোরিপোর জন্য স্ট্যাটিক অ্যানালিসিস। AigisCode পাথ অ্যালিয়াস সমাধান করে, ক্রস-প্যাকেজ ডিপেন্ডেন্সি ট্র্যাক করে এবং আপনার সম্পূর্ণ মনোরিপো জুড়ে সার্কুলার ইমপোর্ট খুঁজে।',
      fr: 'Analyse statique pour les monorepos TypeScript.',
      es: 'Análisis estático para monorepos TypeScript.',
      cs: 'Statická analýza pro TypeScript monorepo.',
    },
    content: {
      en: `<h2 id="monorepo-complexity">The Monorepo Challenge</h2>
<p>TypeScript monorepos are increasingly popular — organizations like Google, Microsoft, Vercel, and Shopify maintain large monorepos with hundreds of packages. But as monorepos grow, architectural issues become harder to spot. A circular dependency between two packages buried deep in the workspace might not cause problems for months until someone tries to extract a package or change the build order. Dead exports that no other package consumes accumulate silently. Barrel files (<code>index.ts</code>) re-export everything, masking which symbols are actually used.</p>
<p>Traditional TypeScript tooling like ESLint operates per-file or per-package. AigisCode analyzes the <strong>entire monorepo</strong> as a single dependency graph, revealing cross-package issues that per-package tools cannot see.</p>

<h2 id="path-alias-resolution">Path Alias Resolution</h2>
<p>Monorepos rely heavily on path aliases to avoid deep relative imports. A <code>tsconfig.json</code> might define <code>@app/*</code> mapping to <code>packages/app/src/*</code>, <code>@shared/*</code> mapping to <code>packages/shared/src/*</code>, and so on. AigisCode reads your path alias configuration and resolves all imports to their actual file paths before building the dependency graph. Configure this in your policy:</p>
<pre><code>{
  "graph": {
    "js_import_aliases": {
      "@app/": "packages/app/src/",
      "@shared/": "packages/shared/src/",
      "@ui/": "packages/ui/src/"
    }
  }
}</code></pre>
<p>With aliases resolved, the dependency graph accurately reflects which files depend on which, even across package boundaries.</p>

<h2 id="cross-package-analysis">Cross-Package Dependency Tracking</h2>
<p>AigisCode builds a unified graph spanning all packages in your monorepo. This enables detection of:</p>
<ul>
<li><strong>Cross-package circular dependencies</strong> — Package A imports from Package B, which imports from Package C, which imports from Package A. This cycle might involve dozens of individual files across three packages.</li>
<li><strong>Bottleneck packages</strong> — Packages that everything depends on. These are coupling hotspots that slow down builds and require extreme care when modifying.</li>
<li><strong>Orphan packages</strong> — Packages that no other package imports from. They might be unused workspace members that can be removed.</li>
<li><strong>Layer violations</strong> — A UI package importing from a server package, or a shared utility importing from an application-specific package.</li>
</ul>

<h2 id="barrel-file-analysis">Barrel File Analysis</h2>
<p>Barrel files (<code>index.ts</code> files that re-export from internal modules) are ubiquitous in TypeScript monorepos. They simplify import paths but hide which symbols are actually consumed by external code. AigisCode traces through barrel files to determine which internal symbols have real consumers and which are re-exported but never imported by any external file. Dead barrel exports inflate your public API surface and confuse consumers of your package.</p>

<h2 id="real-world-example">Real-World Example</h2>
<p>A monorepo with 200+ packages and 1,500 TypeScript files:</p>
<pre><code>$ aigiscode analyze /path/to/monorepo
  Indexing...     1,547 files parsed
  Graphing...     12,834 dependencies mapped

  Results:
  ├── 5 strong circular dependencies (cross-package)
  ├── 89 unused imports
  ├── 23 dead barrel exports
  ├── 12 orphan files
  ├── 3 bottleneck packages (imported by 50+ other packages)
  └── 2 layer violations (UI → Server imports)</code></pre>
<p>The cross-package cycles are the most actionable findings. Each cycle report includes the full import chain, making it clear exactly which imports need to be refactored. The layer violations highlight architectural drift that should be caught before it spreads further.</p>

<h2 id="performance">Performance at Scale</h2>
<p>AigisCode is designed to handle large codebases efficiently. The indexer uses tree-sitter for fast parsing (processing 1,500 TypeScript files in under 10 seconds on modern hardware). The graph builder uses petgraph's optimized algorithms for cycle detection and coupling analysis. For the largest monorepos, you can run AigisCode on specific workspace paths to analyze subsets of the graph.</p>`,
      zh: `<h2 id="monorepo-complexity">Monorepo的挑战</h2>
<p>TypeScript monorepo中，跨包的架构问题更难发现。AigisCode将整个monorepo分析为单一依赖图，解析路径别名、追踪跨包依赖、分析barrel文件并检测层次违规。它能发现传统逐包工具无法看到的跨包循环依赖和瓶颈包。</p>`,
      fr: `<h2 id="monorepo-complexity">Le défi du monorepo</h2>
<p>AigisCode analyse l'ensemble du monorepo comme un seul graphe de dépendances, résolvant les alias de chemins, suivant les dépendances inter-packages et analysant les fichiers barrel. Il détecte les dépendances circulaires inter-packages, les packages goulots d'étranglement et les violations de couches.</p>`,
      es: `<h2 id="monorepo-complexity">El desafío del monorepo</h2>
<p>AigisCode analiza todo el monorepo como un solo grafo de dependencias, resolviendo alias de rutas, rastreando dependencias entre paquetes y analizando archivos barrel. Detecta dependencias circulares entre paquetes, paquetes cuello de botella y violaciones de capas.</p>`,
      cs: `<h2 id="monorepo-complexity">Výzva monorepo</h2>
<p>AigisCode analyzuje celé monorepo jako jediný graf závislostí, rozlišuje aliasy cest, sleduje závislosti mezi balíčky a analyzuje barrel soubory. Detekuje cyklické závislosti mezi balíčky, úzká hrdla balíčků a porušení vrstev.</p>`,
      hi: `<h2 id="monorepo-complexity">Monorepo की चुनौती</h2>
<p>AigisCode पूरे monorepo का एक सिंगल डिपेंडेंसी ग्राफ़ के रूप में विश्लेषण करता है, पाथ एलियास रिज़ॉल्व करता है, क्रॉस-पैकेज डिपेंडेंसी ट्रैक करता है और barrel फ़ाइलों का विश्लेषण करता है। क्रॉस-पैकेज सर्कुलर डिपेंडेंसी, बॉटलनेक पैकेज और लेयर वायलेशन डिटेक्ट करता है।</p>`,
      pt: `<h2 id="monorepo-complexity">O desafio do monorepo</h2>
<p>O AigisCode analisa todo o monorepo como um unico grafo de dependencias, resolvendo aliases de caminhos, rastreando dependencias entre pacotes e analisando arquivos barrel. Detecta dependencias circulares entre pacotes, pacotes gargalo e violacoes de camadas.</p>`,
      ar: `<h2 id="monorepo-challenge">تحدي المستودع الأحادي</h2>
<p>مستودعات TypeScript الأحادية تشمل مئات الحزم مع أسماء مسارات مستعارة وملفات barrel وتبعيات عبر الحزم. يحل AigisCode أسماء المسارات المستعارة ويتتبع التبعيات عبر الحزم ويحلل ملفات barrel للعثور على المشكلات المعمارية عبر المستودع بالكامل.</p>`,
      pl: `<h2 id="monorepo">Monorepo TypeScript</h2>
<p>Rozwiazywanie aliasow sciezek, sledzenie zaleznosci miedzy pakietami, obsluga plikow deklaracji. Cykliczne zaleznosci miedzy pakietami raportowane oddzielnie.</p>`,
    },
    features: [
      'Path alias resolution',
      'Cross-package cycle detection',
      'Barrel file analysis',
      'Bottleneck package identification',
      'Layer violation detection',
      'tree-sitter fast parsing',
    ],
    codeExample: `# Analyze entire monorepo
aigiscode analyze /path/to/monorepo

# Configure path aliases in policy
cat > .aigiscode/policy.json << EOF
{
  "graph": {
    "js_import_aliases": {
      "@app/": "packages/app/src/",
      "@shared/": "packages/shared/src/"
    }
  }
}
EOF

# Check cross-package cycles
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.strong_circular_dependencies'`,
    relatedSlugs: ['circular-dependency-detection', 'dead-code-detection', 'ci-cd-integration'],
  },
  /* ---- 9. CI/CD Integration ---- */
  {
    slug: 'ci-cd-integration',
    icon: 'GitBranch',
    category: 'Workflow',
    title: {
      en: 'CI/CD Pipeline Integration',
      zh: 'CI/CD流水线集成',
      hi: 'CI/CD पाइपलाइन इंटीग्रेशन',
      pt: 'Integração com Pipeline CI/CD',
      ar: 'التكامل مع خطوط أنابيب CI/CD',
      pl: 'Integracja z pipeline CI/CD',
      bn: 'CI/CD Pipeline ইন্টিগ্রেশন',
      fr: 'Intégration Pipeline CI/CD',
      es: 'Integración con Pipeline CI/CD',
      cs: 'Integrace do CI/CD Pipeline',
    },
    shortDescription: {
      en: 'Run AigisCode in your CI/CD pipeline to catch architectural regressions on every pull request. Automate code quality checks with JSON output, exit codes, and deterministic mode.',
      zh: '在CI/CD流水线中运行AigisCode，在每个拉取请求中捕获架构回归。',
      hi: 'हर पुल रिक्वेस्ट पर आर्किटेक्चरल रिग्रेशन पकड़ने के लिए CI/CD पाइपलाइन में AigisCode चलाएं।',
      pt: 'Execute AigisCode em seu pipeline CI/CD para capturar regressões arquiteturais em cada pull request.',
      ar: 'شغّل AigisCode في خط أنابيب CI/CD لاكتشاف الانحدارات المعمارية في كل طلب سحب. أتمت فحوصات جودة الشيفرة بمخرجات JSON وأكواد خروج ووضع حتمي.',
      pl: 'Uruchamiaj AigisCode w swoim pipeline CI/CD, aby wychwytywac regresje architektoniczne przy kazdym pull requeście. Zautomatyzowana analiza z kodami wyjscia, artefaktami JSON i flaga --skip-ai dla deterministycznego CI.',
      bn: 'প্রতিটি pull request-এ আর্কিটেকচারাল রিগ্রেশন ধরতে আপনার CI/CD pipeline-এ AigisCode চালান। JSON আউটপুট, এক্সিট কোড এবং ডিটার্মিনিস্টিক মোড সহ কোড কোয়ালিটি চেক স্বয়ংক্রিয় করুন।',
      fr: 'Exécutez AigisCode dans votre pipeline CI/CD pour détecter les régressions architecturales à chaque pull request.',
      es: 'Ejecute AigisCode en su pipeline CI/CD para capturar regresiones arquitectónicas en cada pull request.',
      cs: 'Spusťte AigisCode ve vašem CI/CD pipeline pro zachycení architektonických regresí v každém pull requestu.',
    },
    metaDescription: {
      en: 'Integrate AigisCode into GitHub Actions, GitLab CI, or any CI/CD pipeline. Automate static analysis with JSON output, exit codes, and deterministic analysis mode.',
      zh: '将AigisCode集成到GitHub Actions、GitLab CI或任何CI/CD流水线中。',
      hi: 'AigisCode को GitHub Actions, GitLab CI या किसी भी CI/CD पाइपलाइन में इंटीग्रेट करें।',
      pt: 'Integre AigisCode ao GitHub Actions, GitLab CI ou qualquer pipeline CI/CD.',
      ar: 'ادمج AigisCode في GitHub Actions أو GitLab CI أو أي خط أنابيب CI/CD. أتمت التحليل الثابت بمخرجات JSON وأكواد خروج ووضع تحليل حتمي.',
      pl: 'Zintegruj AigisCode z GitHub Actions, GitLab CI lub dowolnym pipeline CI/CD. Zautomatyzuj analize statyczna z kodami wyjscia, artefaktami JSON i analiza deterministyczna.',
      bn: 'GitHub Actions, GitLab CI বা যেকোনো CI/CD pipeline-এ AigisCode ইন্টিগ্রেট করুন। JSON আউটপুট, এক্সিট কোড এবং ডিটার্মিনিস্টিক বিশ্লেষণ মোড সহ স্ট্যাটিক অ্যানালিসিস স্বয়ংক্রিয় করুন।',
      fr: 'Intégrez AigisCode dans GitHub Actions, GitLab CI ou tout pipeline CI/CD.',
      es: 'Integre AigisCode en GitHub Actions, GitLab CI o cualquier pipeline CI/CD.',
      cs: 'Integrujte AigisCode do GitHub Actions, GitLab CI nebo jakéhokoli CI/CD pipeline.',
    },
    content: {
      en: `<h2 id="why-ci">Why Run Analysis in CI?</h2>
<p>The most effective time to catch architectural issues is before they merge. A circular dependency introduced in a pull request is trivial to fix — the developer still has the context. The same cycle discovered six months later during a refactoring effort requires archaeology to understand and surgery to remove. AigisCode's JSON output and exit codes make it straightforward to integrate into any CI/CD pipeline.</p>

<h2 id="github-actions">GitHub Actions Integration</h2>
<p>Here is a complete GitHub Actions workflow that runs AigisCode on every pull request:</p>
<pre><code>name: AigisCode Analysis
on:
  pull_request:
    branches: [main]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'

      - name: Install AigisCode
        run: cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode

      - name: Run Analysis
        run: aigiscode analyze .

      - name: Upload Report
        uses: actions/upload-artifact@v4
        with:
          name: aigiscode-report
          path: .aigiscode/deterministic-analysis.json

      - name: Check for Critical Issues
        run: |
          CYCLES=$(cat .aigiscode/deterministic-analysis.json | python3 -c "
          import json, sys
          data = json.load(sys.stdin)
          cycles = data.get('graph_analysis', {}).get('strong_circular_dependencies', [])
          print(len(cycles))
          ")
          if [ "$CYCLES" -gt 0 ]; then
            echo "::error::Found $CYCLES strong circular dependencies"
            exit 1
          fi</code></pre>
<p>This workflow installs AigisCode, runs deterministic analysis (no AI API costs), uploads the report as an artifact, and fails the build if any strong circular dependencies are found.</p>

<h2 id="gitlab-ci">GitLab CI Integration</h2>
<p>For GitLab CI, add this to your <code>.gitlab-ci.yml</code>:</p>
<pre><code>aigiscode:
  stage: test
  image: rust:1.77-slim
  script:
    - cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
    - aigiscode analyze .
    - python3 -c "
      import json;
      data = json.load(open('.aigiscode/deterministic-analysis.json'));
      cycles = data.get('graph_analysis', {}).get('strong_circular_dependencies', []);
      dead = data.get('dead_code', []);
      print(f'Cycles: {len(cycles)}, Dead code items: {len(dead)}');
      exit(1 if len(cycles) > 0 else 0)
      "
  artifacts:
    paths:
      - .aigiscode/deterministic-analysis.json
    when: always</code></pre>

<h2 id="deterministic-mode">Deterministic Mode for CI</h2>
<p>In CI environments, use the <code>--skip-ai</code> flag to run AigisCode in fully deterministic mode. This provides several advantages:</p>
<ul>
<li><strong>No API keys required</strong> — No need to configure OpenAI or Anthropic secrets in your CI environment.</li>
<li><strong>Zero cost</strong> — No API calls means no usage charges.</li>
<li><strong>Reproducible results</strong> — The same code always produces the same findings. This is essential for CI where you want consistent pass/fail behavior.</li>
<li><strong>Faster execution</strong> — Skipping the AI review stage reduces analysis time by 30-60%.</li>
</ul>

<h2 id="exit-codes">Exit Codes and JSON Output</h2>
<p>AigisCode produces a structured JSON report at <code>.aigiscode/deterministic-analysis.json</code>. You can parse this report in your CI script to implement custom quality gates. Common patterns include:</p>
<ul>
<li>Fail the build if <strong>any</strong> strong circular dependencies exist</li>
<li>Fail if dead code count exceeds a threshold (gradual improvement approach)</li>
<li>Warn on new hardwired values but do not block the merge</li>
<li>Track finding counts over time to detect trends</li>
</ul>

<h2 id="baseline-approach">The Baseline Approach</h2>
<p>For existing projects with many findings, we recommend the baseline approach:</p>
<ol>
<li>Run AigisCode on <code>main</code> to generate a baseline report.</li>
<li>Store the baseline finding counts in a configuration file.</li>
<li>On each PR, run AigisCode and compare against the baseline.</li>
<li>Fail the build only if new findings are introduced (count increased).</li>
<li>Gradually reduce the baseline as the team fixes existing issues.</li>
</ol>
<p>This approach lets you adopt AigisCode without blocking all PRs on day one, while still preventing architectural debt from growing.</p>

<h2 id="pr-comments">Pull Request Comments</h2>
<p>For an even better developer experience, you can parse the AigisCode report and post a summary as a PR comment. The JSON report structure is stable and documented, making it straightforward to build a custom integration that posts findings directly on the pull request for easy review.</p>`,
      zh: `<h2 id="why-ci">为什么在CI中运行分析</h2>
<p>在合并前捕获架构问题是最有效的时机。AigisCode的JSON输出和退出码使其易于集成到任何CI/CD流水线中。使用--skip-ai标志进行完全确定性分析，无需API密钥或费用。支持GitHub Actions和GitLab CI，可以设置自定义质量门控。</p>`,
      fr: `<h2 id="why-ci">Pourquoi exécuter l'analyse en CI</h2>
<p>Détecter les problèmes architecturaux avant la fusion est le moment le plus efficace. La sortie JSON et les codes de sortie d'AigisCode facilitent l'intégration dans tout pipeline CI/CD. Utilisez --skip-ai pour une analyse déterministe sans clés API ni coûts. Supporte GitHub Actions et GitLab CI.</p>`,
      es: `<h2 id="why-ci">Por qué ejecutar análisis en CI</h2>
<p>Detectar problemas arquitectónicos antes de la fusión es el momento más efectivo. La salida JSON y los códigos de salida de AigisCode facilitan la integración en cualquier pipeline CI/CD. Use --skip-ai para análisis determinístico sin claves API ni costos. Soporta GitHub Actions y GitLab CI.</p>`,
      cs: `<h2 id="why-ci">Proč spouštět analýzu v CI</h2>
<p>Zachycení architektonických problémů před sloučením je nejefektivnější. JSON výstup a návratové kódy AigisCode usnadňují integraci do jakéhokoli CI/CD pipeline. Použijte --skip-ai pro deterministickou analýzu bez API klíčů nebo nákladů. Podporuje GitHub Actions a GitLab CI.</p>`,
      hi: `<h2 id="why-ci">CI में एनालिसिस क्यों चलाएं</h2>
<p>मर्ज से पहले आर्किटेक्चरल समस्याएं पकड़ना सबसे प्रभावी समय है। AigisCode का JSON आउटपुट और एग्ज़िट कोड इसे किसी भी CI/CD पाइपलाइन में इंटीग्रेट करना आसान बनाते हैं। API कुंजी या लागत के बिना डिटर्मिनिस्टिक एनालिसिस के लिए --skip-ai का उपयोग करें। GitHub Actions और GitLab CI सपोर्ट करता है।</p>`,
      pt: `<h2 id="why-ci">Por que executar analise em CI</h2>
<p>Capturar problemas arquiteturais antes do merge e o momento mais eficaz. A saida JSON e os codigos de saida do AigisCode facilitam a integracao em qualquer pipeline CI/CD. Use --skip-ai para analise deterministica sem chaves API ou custos. Suporta GitHub Actions e GitLab CI.</p>`,
      ar: `<h2 id="why-ci">لماذا تشغّل التحليل في CI</h2>
<p>التحليل الثابت أكثر فعالية عند تشغيله تلقائياً. ادمج AigisCode في خط أنابيب CI لاكتشاف التبعيات الدائرية والشيفرة الميتة والقيم الثابتة قبل الدمج. يدعم أكواد الخروج ومخرجات JSON وعلامة <code>--skip-ai</code> للتحليل الحتمي.</p>`,
      pl: `<h2 id="ci-cd">Integracja CI/CD</h2>
<p>Kody wyjscia, artefakty JSON, flaga <code>--skip-ai</code>. Dziala z GitHub Actions, GitLab CI i dowolnym systemem CI.</p>`,
    },
    features: [
      'GitHub Actions workflow',
      'GitLab CI configuration',
      'Deterministic --skip-ai mode',
      'JSON report output',
      'Custom quality gates',
      'Baseline comparison approach',
    ],
    codeExample: `# Quick CI integration (any platform)
cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
aigiscode analyze .

# Check for critical issues
python3 -c "
import json
data = json.load(open('.aigiscode/deterministic-analysis.json'))
cycles = data.get('graph_analysis', {}).get('strong_circular_dependencies', [])
print(f'Found {len(cycles)} strong circular dependencies')
exit(1 if cycles else 0)
"`,
    relatedSlugs: ['ai-assisted-code-review', 'typescript-monorepo-analysis', 'python-codebase-analysis'],
  },
  /* ---- 10. Python Codebase Analysis ---- */
  {
    slug: 'python-codebase-analysis',
    icon: 'FileCode',
    category: 'Workflow',
    title: {
      en: 'Python Codebase Analysis',
      zh: 'Python代码库分析',
      hi: 'Python कोडबेस एनालिसिस',
      pt: 'Análise de Codebase Python',
      ar: 'تحليل قاعدة شيفرة Python',
      pl: 'Analiza baz kodu Python',
      bn: 'Python কোডবেস বিশ্লেষণ',
      fr: 'Analyse de Codebase Python',
      es: 'Análisis de Codebase Python',
      cs: 'Analýza Python Kódové Báze',
    },
    shortDescription: {
      en: 'Analyze Python projects for dead imports, circular module dependencies, and hardwired configuration values. AigisCode uses tree-sitter-python for precise parsing and supports both flat and package-based layouts.',
      zh: '分析Python项目的死导入、循环模块依赖和硬编码配置值。',
      hi: 'डेड इम्पोर्ट्स, सर्कुलर मॉड्यूल डिपेंडेंसीज और हार्डवायर्ड कॉन्फ़िगरेशन वैल्यूज के लिए Python प्रोजेक्ट्स का विश्लेषण करें।',
      pt: 'Analise projetos Python para imports mortos, dependências circulares e valores de configuração hardcoded.',
      ar: 'تحليل مشاريع Python للاستيرادات الميتة وتبعيات الوحدات الدائرية وقيم التكوين الثابتة. يستخدم AigisCode محلل tree-sitter-python للتحليل الدقيق ويدعم التخطيطات المسطحة والقائمة على الحزم.',
      pl: 'Analizuj projekty Python pod katem martwych importow, cyklicznych zaleznosci modulow i zakodowanych wartosci konfiguracyjnych. Uzywa natywnego modulu AST Pythona do dokladnego parsowania bez zaleznosci.',
      bn: 'ডেড ইমপোর্ট, সার্কুলার মডিউল ডিপেন্ডেন্সি এবং হার্ডওয়্যার্ড কনফিগারেশন ভ্যালুর জন্য Python প্রজেক্ট বিশ্লেষণ করুন। AigisCode সুনির্দিষ্ট পার্সিংয়ের জন্য tree-sitter-python ব্যবহার করে এবং ফ্ল্যাট ও প্যাকেজ-ভিত্তিক উভয় লেআউট সাপোর্ট করে।',
      fr: 'Analysez les projets Python pour les imports morts, les dépendances circulaires et les valeurs de configuration en dur.',
      es: 'Analice proyectos Python para imports muertos, dependencias circulares y valores de configuración hardcodeados.',
      cs: 'Analyzujte Python projekty pro mrtvé importy, cyklické závislosti modulů a natvrdo zadané konfigurační hodnoty.',
    },
    metaDescription: {
      en: 'Static analysis for Python codebases. Find dead imports, circular module dependencies, hardwired values, and god classes in Python projects using AST-based analysis.',
      zh: 'Python代码库的静态分析。',
      hi: 'Python कोडबेस के लिए स्टैटिक एनालिसिस।',
      pt: 'Análise estática para codebases Python.',
      ar: 'تحليل ثابت لقواعد شيفرة Python. اعثر على الاستيرادات الميتة وتبعيات الوحدات الدائرية والقيم الثابتة والفئات الضخمة في مشاريع Python باستخدام تحليل مبني على AST.',
      pl: 'Analiza statyczna dla baz kodu Python. Znajdz martwe importy, cykliczne zaleznosci modulow, zakodowane wartosci i problemy architektoniczne z natywnym parsowaniem AST.',
      bn: 'Python কোডবেসের জন্য স্ট্যাটিক অ্যানালিসিস। AST-ভিত্তিক বিশ্লেষণ ব্যবহার করে Python প্রজেক্টে ডেড ইমপোর্ট, সার্কুলার মডিউল ডিপেন্ডেন্সি, হার্ডওয়্যার্ড ভ্যালু এবং গড ক্লাস খুঁজুন।',
      fr: 'Analyse statique pour les codebases Python.',
      es: 'Análisis estático para codebases Python.',
      cs: 'Statická analýza pro Python kódové báze.',
    },
    content: {
      en: `<h2 id="python-specific-challenges">Python-Specific Analysis Challenges</h2>
<p>Python's dynamic nature makes static analysis both valuable and challenging. Import statements can appear anywhere in a file — at the top level, inside functions (lazy imports), or inside conditional blocks. Modules can be imported by name via <code>importlib.import_module()</code>. Monkey-patching can change what a module exports at runtime. Despite these challenges, the vast majority of Python code follows conventional patterns that AigisCode can analyze accurately.</p>
<p>AigisCode uses Python's own AST module for parsing Python files, ensuring perfect fidelity with the language's syntax. This is more reliable than regex-based approaches and handles all Python syntax variations correctly, including f-strings, walrus operators, match statements, and type annotations.</p>

<h2 id="dead-imports">Dead Import Detection</h2>
<p>Dead imports are the most common form of dead code in Python. They accumulate naturally as code evolves — a function is refactored to no longer use a particular module, but the import statement remains. Python's import system executes imported modules as a side effect, meaning dead imports can slow down application startup, increase memory usage, and create unnecessary coupling.</p>
<p>AigisCode detects several types of dead imports:</p>
<ul>
<li><strong>Completely unused imports</strong> — <code>import os</code> where <code>os</code> is never referenced in the file.</li>
<li><strong>Partially unused from-imports</strong> — <code>from typing import List, Dict, Optional</code> where only <code>Optional</code> is actually used.</li>
<li><strong>Star import analysis</strong> — <code>from module import *</code> imports that bring in symbols never referenced.</li>
<li><strong>Conditional import tracking</strong> — Imports inside <code>if TYPE_CHECKING:</code> blocks are handled correctly as type-only imports.</li>
</ul>

<h2 id="circular-modules">Circular Module Dependencies</h2>
<p>Python's import system can handle some circular imports at runtime through deferred resolution, but circular module dependencies still cause real problems:</p>
<ul>
<li><strong>ImportError at startup</strong> — If Module A imports a name from Module B at the top level, and Module B imports from Module A, Python raises <code>ImportError</code> because the name has not been defined yet when the circular import executes.</li>
<li><strong>Unpredictable initialization order</strong> — Module-level code (class definitions, global variables) may execute in a different order than expected.</li>
<li><strong>Testing difficulties</strong> — Circular dependencies make it impossible to import and test a module in isolation.</li>
</ul>
<p>AigisCode's graph analyzer maps the full import graph of your Python project and identifies both strong and weak cycles. Strong cycles (mutual top-level imports) are high priority. Weak cycles (where one side uses a lazy import inside a function) are lower priority but still reported.</p>

<h2 id="hardwired-config">Hardwired Configuration Detection</h2>
<p>Python applications frequently scatter configuration across the codebase: database URLs in utility modules, API keys in middleware files, timeouts in service classes. AigisCode flags these patterns:</p>
<pre><code># Flagged: hardcoded database URL outside config
db = create_engine("postgresql://user:pass@localhost:5432/mydb")

# Flagged: scattered os.environ access
api_key = os.environ.get("STRIPE_API_KEY")  # in payments/service.py

# Flagged: magic number
MAX_RETRIES = 3  # repeated in 5 different files</code></pre>
<p>The recommended fix is centralizing configuration in a dedicated settings module (like Django's <code>settings.py</code> or a <code>config.py</code> using Serde settings) and importing values from there.</p>

<h2 id="real-world-example">Real-World Example</h2>
<p>A Python web application with 300 files:</p>
<pre><code>$ aigiscode analyze /path/to/python-app
  Results:
  ├── 4 strong circular dependencies
  ├── 34 unused imports across 22 files
  ├── 8 partially unused from-imports
  ├── 6 hardcoded database/API URLs
  ├── 11 os.environ calls outside config module
  ├── 5 orphan utility modules
  └── 1 god class (BaseService: 41 methods)</code></pre>
<p>The 34 unused imports alone represent measurable startup time savings when removed. The circular dependencies explain why certain test suites fail with import errors when run in isolation.</p>

<h2 id="type-checking">Type Checking Integration</h2>
<p>AigisCode correctly handles <code>TYPE_CHECKING</code> blocks from the <code>typing</code> module. Imports inside <code>if TYPE_CHECKING:</code> are treated as type-only references and are not flagged as unused even if they are not used at runtime. This ensures compatibility with mypy and pyright type checking workflows.</p>

<h2 id="virtual-environments">Virtual Environment Handling</h2>
<p>AigisCode automatically excludes virtual environment directories (<code>venv/</code>, <code>.venv/</code>, <code>env/</code>) and common non-source directories (<code>__pycache__/</code>, <code>.eggs/</code>, <code>dist/</code>) from analysis. Only your application code is analyzed, keeping the report focused and the analysis fast.</p>`,
      zh: `<h2 id="python-specific-challenges">Python特定分析挑战</h2>
<p>Python的动态特性使静态分析既有价值又具有挑战性。AigisCode使用Python的AST模块进行精确解析，检测完全未使用的导入、部分未使用的from导入、星号导入和TYPE_CHECKING块。它映射完整的导入图来识别循环模块依赖，并标记配置模块外的硬编码值。</p>`,
      fr: `<h2 id="python-specific-challenges">Défis spécifiques à Python</h2>
<p>AigisCode utilise le module AST de Python pour un parsing précis. Il détecte les imports inutilisés, les imports partiellement utilisés, les dépendances circulaires entre modules et les valeurs de configuration en dur. Les blocs TYPE_CHECKING sont gérés correctement pour la compatibilité avec mypy et pyright.</p>`,
      es: `<h2 id="python-specific-challenges">Desafíos específicos de Python</h2>
<p>AigisCode utiliza el módulo AST de Python para un parsing preciso. Detecta imports no utilizados, imports parcialmente utilizados, dependencias circulares entre módulos y valores de configuración hardcodeados. Los bloques TYPE_CHECKING se manejan correctamente para compatibilidad con mypy y pyright.</p>`,
      cs: `<h2 id="python-specific-challenges">Python specifické výzvy analýzy</h2>
<p>AigisCode používá modul AST Pythonu pro přesný parsing. Detekuje nepoužívané importy, částečně nepoužívané importy, cyklické závislosti modulů a natvrdo zadané konfigurační hodnoty. Bloky TYPE_CHECKING jsou správně zpracovány pro kompatibilitu s mypy a pyright.</p>`,
      hi: `<h2 id="python-specific-challenges">Python विशिष्ट एनालिसिस चुनौतियां</h2>
<p>AigisCode सटीक पार्सिंग के लिए Python के AST मॉड्यूल का उपयोग करता है। अनुपयोगी इम्पोर्ट, आंशिक रूप से अनुपयोगी इम्पोर्ट, सर्कुलर मॉड्यूल डिपेंडेंसी और हार्डकोडेड कॉन्फ़िगरेशन वैल्यू डिटेक्ट करता है। mypy और pyright के साथ कम्पैटिबिलिटी के लिए TYPE_CHECKING ब्लॉक सही ढंग से हैंडल होते हैं।</p>`,
      pt: `<h2 id="python-specific-challenges">Desafios especificos de analise Python</h2>
<p>O AigisCode usa o modulo AST do Python para analise precisa. Detecta imports nao utilizados, imports parcialmente utilizados, dependencias circulares entre modulos e valores de configuracao hardcoded. Blocos TYPE_CHECKING sao tratados corretamente para compatibilidade com mypy e pyright.</p>`,
      ar: `<h2 id="python-challenges">تحديات التحليل الخاصة بـ Python</h2>
<p>تواجه قواعد شيفرة Python تحديات فريدة: الاستيرادات الدائرية وعلامات الأنواع والبيئات الافتراضية والتحميل الديناميكي. يستخدم AigisCode وحدة AST الأصلية في Python للتحليل الدقيق ويدعم التخطيطات المسطحة والقائمة على الحزم مع تعرّف أنماط إطار Django.</p>`,
      pl: `<h2 id="python">Analiza Python</h2>
<p>Natywny modul AST. Martwe importy, cykliczne zaleznosci modulow, zakodowane wartosci. Zero zewnetrznych zaleznosci parsera.</p>`,
    },
    features: [
      'tree-sitter-python parsing',
      'Dead import detection',
      'Partial from-import analysis',
      'TYPE_CHECKING compatibility',
      'Circular module detection',
      'Virtual env exclusion',
    ],
    codeExample: `# Analyze a Python project
aigiscode analyze /path/to/python-project

# View dead imports
cat .aigiscode/deterministic-analysis.json | jq '.dead_code[] | select(.type == "unused_import")'

# View circular module dependencies
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.strong_circular_dependencies'`,
    relatedSlugs: ['dead-code-detection', 'django-project-analysis', 'ci-cd-integration'],
  },
];

/* -------------------------------------------------------------------------- */
/*  Helpers                                                                   */
/* -------------------------------------------------------------------------- */

export const categories = [...new Set(useCases.map((uc) => uc.category))];

export function getUseCaseBySlug(slug: string): UseCase | undefined {
  return useCases.find((uc) => uc.slug === slug);
}

export function getRelatedUseCases(slugs: string[]): UseCase[] {
  return slugs.map((s) => useCases.find((uc) => uc.slug === s)).filter(Boolean) as UseCase[];
}
