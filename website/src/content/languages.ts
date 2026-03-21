/* -------------------------------------------------------------------------- */
/*  Language Data — AigisCode Marketing Website                              */
/* -------------------------------------------------------------------------- */

export interface Language {
  slug: string;
  icon: string;
  title: Record<string, string>;
  shortDescription: Record<string, string>;
  metaDescription: Record<string, string>;
  parser: string;
  extensions: string;
  features: string[];
  detectorSupport: {
    deadCode: boolean;
    hardwiring: boolean;
    graphAnalysis: boolean;
  };
  frameworkPlugins: string[];
  content: Record<string, string>;
  relatedSlugs: string[];
}

export const languages: Language[] = [
  /* ---- 1. PHP ---- */
  {
    slug: 'php',
    icon: 'FilePhp',
    title: {
      en: 'PHP Analysis',
      cs: 'Analýza PHP',
      fr: 'Analyse PHP',
      es: 'Análisis de PHP',
      zh: 'PHP 分析',
      hi: 'PHP विश्लेषण',
      pt: 'Análise de PHP',
      ar: 'تحليل PHP',
      pl: 'Analiza PHP',
      bn: 'PHP বিশ্লেষণ',
    },
    shortDescription: {
      en: 'Deep structural analysis of PHP codebases using tree-sitter parsing. Detects circular dependencies between classes, dead methods, unused imports, hardwired configuration values, and namespace violations across your entire project.',
      cs: 'Hloubková strukturální analýza PHP kódových bází pomocí tree-sitter parseru. Detekuje cyklické závislosti mezi třídami, mrtvé metody, nepoužívané importy, natvrdo zapsané konfigurační hodnoty a porušení jmenných prostorů v celém projektu.',
      fr: 'Analyse structurelle approfondie des bases de code PHP avec le parseur tree-sitter. Détecte les dépendances circulaires entre classes, les méthodes mortes, les imports inutilisés, les valeurs de configuration codées en dur et les violations de namespaces dans l\'ensemble du projet.',
      es: 'Análisis estructural profundo de bases de código PHP utilizando el parser tree-sitter. Detecta dependencias circulares entre clases, métodos muertos, imports no utilizados, valores de configuración codificados y violaciones de namespaces en todo el proyecto.',
      zh: '使用 tree-sitter 解析器对 PHP 代码库进行深度结构分析。检测类之间的循环依赖、死方法、未使用的 import、硬编码的配置值以及整个项目中的命名空间违规。',
      hi: 'tree-sitter पार्सर का उपयोग करके PHP कोडबेस का गहन संरचनात्मक विश्लेषण। कक्षाओं के बीच चक्रीय निर्भरता, मृत विधियाँ, अप्रयुक्त import, हार्डकोड किए गए कॉन्फ़िगरेशन मान और पूरे प्रोजेक्ट में namespace उल्लंघनों का पता लगाता है।',
      pt: 'Análise estrutural profunda de bases de código PHP usando o parser tree-sitter. Detecta dependências circulares entre classes, métodos mortos, imports não utilizados, valores de configuração codificados e violações de namespaces em todo o projeto.',
      ar: 'تحليل هيكلي عميق لقواعد شيفرة PHP باستخدام محلل tree-sitter. يكتشف التبعيات الدائرية بين الفئات والطرق الميتة والاستيرادات غير المستخدمة وقيم التكوين الثابتة وانتهاكات فضاء الأسماء عبر مشروعك بالكامل.',
      pl: 'Gleboka analiza strukturalna baz kodu PHP z parsowaniem tree-sitter. Wykrywa cykliczne zaleznosci miedzy klasami, martwe metody, nieuzywane importy, zakodowane wartosci konfiguracyjne i naruszenia przestrzeni nazw w calym projekcie.',
      bn: 'tree-sitter পার্সিং ব্যবহার করে PHP কোডবেসের গভীর কাঠামোগত বিশ্লেষণ। আপনার সম্পূর্ণ প্রজেক্ট জুড়ে ক্লাসের মধ্যে সার্কুলার ডিপেন্ডেন্সি, ডেড মেথড, অব্যবহৃত ইমপোর্ট, হার্ডওয়্যার্ড কনফিগারেশন ভ্যালু এবং নেমস্পেস ভায়োলেশন শনাক্ত করে।',
    },
    metaDescription: {
      en: 'Analyze PHP codebases for circular dependencies, dead code, hardwired values, and architectural issues. AigisCode uses tree-sitter to parse classes, functions, namespaces, traits, and imports.',
      cs: 'Analyzujte PHP kódové báze na cyklické závislosti, mrtvý kód, natvrdo zapsané hodnoty a architektonické problémy. AigisCode používá tree-sitter k parsování tříd, funkcí, jmenných prostorů, traitů a importů.',
      fr: 'Analysez les bases de code PHP pour détecter les dépendances circulaires, le code mort, les valeurs codées en dur et les problèmes architecturaux. AigisCode utilise tree-sitter pour parser les classes, fonctions, namespaces, traits et imports.',
      es: 'Analice bases de código PHP en busca de dependencias circulares, código muerto, valores codificados y problemas arquitectónicos. AigisCode utiliza tree-sitter para analizar clases, funciones, namespaces, traits e imports.',
      zh: '分析 PHP 代码库中的循环依赖、死代码、硬编码值和架构问题。AigisCode 使用 tree-sitter 解析类、函数、命名空间、trait 和 import。',
      hi: 'चक्रीय निर्भरता, मृत कोड, हार्डकोड किए गए मान और आर्किटेक्चर संबंधी समस्याओं के लिए PHP कोडबेस का विश्लेषण करें। AigisCode कक्षाओं, फ़ंक्शन, namespace, trait और import को पार्स करने के लिए tree-sitter का उपयोग करता है।',
      pt: 'Analise bases de código PHP para dependências circulares, código morto, valores codificados e problemas arquiteturais. AigisCode usa tree-sitter para analisar classes, funções, namespaces, traits e imports.',
      ar: 'تحليل قواعد شيفرة PHP للتبعيات الدائرية والشيفرة الميتة والقيم الثابتة والمشكلات المعمارية. يستخدم AigisCode محلل tree-sitter لتحليل الفئات والدوال وفضاءات الأسماء والسمات والاستيرادات.',
      pl: 'Analizuj bazy kodu PHP pod katem cyklicznych zaleznosci, martwego kodu, zakodowanych wartosci i problemow architektonicznych. AigisCode uzywa tree-sitter do parsowania klas, funkcji, przestrzeni nazw, traitow i importow.',
      bn: 'সার্কুলার ডিপেন্ডেন্সি, ডেড কোড, হার্ডওয়্যার্ড ভ্যালু এবং আর্কিটেকচারাল সমস্যার জন্য PHP কোডবেস বিশ্লেষণ করুন। AigisCode ক্লাস, ফাংশন, নেমস্পেস, ট্রেইট এবং ইমপোর্ট পার্স করতে tree-sitter ব্যবহার করে।',
    },
    parser: 'tree-sitter',
    extensions: '.php',
    features: [
      'Class and method extraction',
      'Function declarations',
      'Import and use statements',
      'Namespace resolution',
      'Trait detection',
      'Constant and property extraction',
      'PSR-4 autoload mapping',
      'Static method call tracking',
    ],
    detectorSupport: {
      deadCode: true,
      hardwiring: true,
      graphAnalysis: true,
    },
    frameworkPlugins: ['Laravel', 'WordPress'],
    content: {
      en: `<h2 id="how-aigiscode-parses-php">How AigisCode Parses PHP</h2>
<p>AigisCode uses <strong>tree-sitter</strong> to parse PHP source files into a full concrete syntax tree. Unlike regex-based tools that match patterns line by line, tree-sitter builds a complete AST that understands PHP's grammar — nested classes, anonymous functions, complex use statements, and heredoc strings are all parsed correctly.</p>
<p>The parser processes every <code>.php</code> file in your project, extracting a normalized set of symbols that feed into the dependency graph and detector pipeline. Parsing is deterministic and fast — a 500-file Laravel project typically indexes in under 3 seconds.</p>

<h2 id="symbol-extraction">Symbol Extraction</h2>
<p>For each PHP file, AigisCode extracts the following symbols:</p>
<ul>
<li><strong>Classes</strong> — Full class declarations including abstract classes, final classes, and anonymous classes. Parent class and implemented interfaces are tracked as outbound dependencies.</li>
<li><strong>Functions</strong> — Both standalone functions and class methods. Visibility modifiers (public, protected, private) are captured for dead code analysis.</li>
<li><strong>Imports</strong> — All <code>use</code> statements including class imports, function imports, and constant imports. Group use declarations are expanded into individual entries.</li>
<li><strong>Namespaces</strong> — The declared namespace for each file, used to resolve fully qualified class references and build accurate dependency edges.</li>
<li><strong>Traits</strong> — Trait declarations and <code>use TraitName</code> statements inside classes. Trait usage creates dependency edges in the graph.</li>
<li><strong>Constants and Properties</strong> — Class constants and properties are extracted for dead code and hardwiring detection.</li>
</ul>
<p>All symbols are stored in a normalized SQLite index with file paths, line numbers, and symbol types. This index powers every subsequent analysis stage.</p>

<h2 id="framework-support">Framework Support</h2>
<p>AigisCode ships with built-in runtime plugins for PHP frameworks:</p>
<h3 id="laravel-plugin">Laravel</h3>
<p>The Laravel plugin understands Laravel's conventions that a generic analyzer would misclassify:</p>
<ul>
<li><strong>Service providers</strong> — Classes registered in <code>config/app.php</code> or auto-discovered providers are recognized as entry points, not orphan files.</li>
<li><strong>Facades</strong> — Static calls to Facade classes are resolved to their underlying service container bindings where possible.</li>
<li><strong>Eloquent models</strong> — Relationships defined via <code>hasMany</code>, <code>belongsTo</code>, and other methods create implicit dependency edges. String-based model references in foreign keys are not flagged as hardwired values.</li>
<li><strong>Blade views</strong> — Component references in Blade templates (<code>@component</code>, <code>&lt;x-component&gt;</code>) are tracked as dependencies.</li>
<li><strong>Route registration</strong> — Controller references in route files are captured as entry points for dead code analysis.</li>
</ul>
<h3 id="wordpress-plugin">WordPress</h3>
<p>The WordPress plugin handles WordPress's hook-based architecture:</p>
<ul>
<li><strong>Action and filter hooks</strong> — Functions registered via <code>add_action</code> and <code>add_filter</code> are recognized as entry points.</li>
<li><strong>Plugin bootstrap</strong> — Main plugin files and theme <code>functions.php</code> are treated as root entry points.</li>
<li><strong>Shortcodes</strong> — Functions registered via <code>add_shortcode</code> are not flagged as dead code.</li>
</ul>

<h2 id="detection-capabilities">Detection Capabilities</h2>
<p>With PHP fully indexed, AigisCode runs the following detectors:</p>
<ul>
<li><strong>Circular dependencies</strong> — File-level and namespace-level cycles are detected using Tarjan's algorithm on the import graph. The analyzer distinguishes strong architectural cycles (where refactoring one class forces changes in all cycle members) from benign runtime references.</li>
<li><strong>Dead code</strong> — Unused imports, unreferenced private and protected methods, orphaned classes with no inbound dependencies, and abandoned properties are all flagged with confidence levels.</li>
<li><strong>Hardwired values</strong> — Magic strings, repeated string literals across files, hardcoded IP addresses, hardcoded URLs, and direct <code>$_ENV</code> / <code>getenv()</code> access outside config files are detected. The hardwiring detector respects policy exclusions so framework-conventional patterns are not flagged.</li>
<li><strong>God classes</strong> — Classes with an excessive number of methods, properties, or dependencies are identified as architectural hotspots.</li>
<li><strong>Bottleneck files</strong> — Files with abnormally high inbound coupling (many other files depend on them) are flagged so teams can evaluate whether the coupling is intentional.</li>
</ul>

<h2 id="example-findings">Example Findings</h2>
<p>A typical AigisCode report for a PHP project might include:</p>
<pre><code># Run analysis
aigiscode analyze /path/to/laravel-project

# Sample findings from the JSON report:
#
# Strong cycle: app/Services/OrderService.php
#              → app/Services/PaymentService.php
#              → app/Services/OrderService.php
#
# Dead code:   app/Http/Controllers/LegacyApiController.php
#              0 inbound references, confidence: high
#
# Hardwiring:  app/Services/EmailService.php:42
#              Hardcoded URL "https://api.sendgrid.com/v3/mail/send"
#              → Move to config/services.php or .env</code></pre>`,
      cs: `<h2 id="how-aigiscode-parses-php">Jak AigisCode parsuje PHP</h2>
<p>AigisCode používá <strong>tree-sitter</strong> k parsování PHP souborů do kompletního syntaktického stromu. Na rozdíl od nástrojů založených na regulárních výrazech tree-sitter rozumí gramatice PHP — vnořené třídy, anonymní funkce, komplexní use příkazy jsou všechny parsovány správně.</p>

<h2 id="symbol-extraction">Extrakce symbolů</h2>
<p>Pro každý PHP soubor AigisCode extrahuje: třídy (včetně abstraktních a finálních), funkce a metody tříd s modifikátory viditelnosti, importy (<code>use</code> příkazy), jmenné prostory, traity a konstanty/vlastnosti. Všechny symboly jsou uloženy v normalizovaném SQLite indexu.</p>

<h2 id="framework-support">Podpora frameworků</h2>
<p><strong>Laravel</strong> — Plugin rozumí service providers, facades, Eloquent modelům, Blade šablonám a registraci rout. <strong>WordPress</strong> — Plugin zpracovává architekturu založenou na hocích: <code>add_action</code>, <code>add_filter</code>, bootstrap soubory pluginů a shortcodes.</p>

<h2 id="detection-capabilities">Detekční schopnosti</h2>
<p>S plně zaindexovaným PHP AigisCode spouští detektory: cyklické závislosti (Tarjanův algoritmus), mrtvý kód (nepoužívané importy, neodkazované metody, osiřelé třídy), hardwired hodnoty (magické řetězce, natvrdo zapsané IP adresy, přímý přístup k <code>$_ENV</code>), god třídy a soubory úzkých míst.</p>`,
      fr: `<h2 id="how-aigiscode-parses-php">Comment AigisCode analyse PHP</h2>
<p>AigisCode utilise <strong>tree-sitter</strong> pour parser les fichiers PHP en un arbre syntaxique complet. Contrairement aux outils bases sur les regex, tree-sitter comprend la grammaire PHP — classes imbriquees, fonctions anonymes, instructions use complexes sont tous parses correctement.</p>

<h2 id="symbol-extraction">Extraction de symboles</h2>
<p>Pour chaque fichier PHP, AigisCode extrait : classes (abstraites, finales), fonctions et methodes avec modificateurs de visibilite, imports (<code>use</code>), espaces de noms, traits et constantes/proprietes. Tous les symboles sont stockes dans un index SQLite normalise.</p>

<h2 id="framework-support">Support des frameworks</h2>
<p><strong>Laravel</strong> — Le plugin comprend les service providers, facades, modeles Eloquent, templates Blade et l'enregistrement des routes. <strong>WordPress</strong> — Le plugin gere l'architecture basee sur les hooks : <code>add_action</code>, <code>add_filter</code>, fichiers bootstrap de plugins et shortcodes.</p>

<h2 id="detection-capabilities">Capacites de detection</h2>
<p>Avec PHP entierement indexe, AigisCode execute les detecteurs : dependances circulaires (algorithme de Tarjan), code mort (imports inutilises, methodes non referencees, classes orphelines), valeurs codees en dur (chaines magiques, IPs codees en dur, acces direct a <code>$_ENV</code>), classes dieu et fichiers goulots.</p>`,
      es: `<h2 id="how-aigiscode-parses-php">Como AigisCode analiza PHP</h2>
<p>AigisCode usa <strong>tree-sitter</strong> para analizar archivos PHP en un arbol sintactico completo. A diferencia de herramientas basadas en regex, tree-sitter entiende la gramatica PHP — clases anidadas, funciones anonimas, sentencias use complejas se analizan correctamente.</p>

<h2 id="symbol-extraction">Extraccion de simbolos</h2>
<p>Para cada archivo PHP, AigisCode extrae: clases (abstractas, finales), funciones y metodos con modificadores de visibilidad, imports (<code>use</code>), namespaces, traits y constantes/propiedades. Todos los simbolos se almacenan en un indice SQLite normalizado.</p>

<h2 id="framework-support">Soporte de frameworks</h2>
<p><strong>Laravel</strong> — El plugin comprende service providers, facades, modelos Eloquent, plantillas Blade y registro de rutas. <strong>WordPress</strong> — El plugin maneja la arquitectura basada en hooks: <code>add_action</code>, <code>add_filter</code>, archivos bootstrap de plugins y shortcodes.</p>

<h2 id="detection-capabilities">Capacidades de deteccion</h2>
<p>Con PHP completamente indexado, AigisCode ejecuta detectores: dependencias circulares (algoritmo de Tarjan), codigo muerto (imports no utilizados, metodos no referenciados, clases huerfanas), valores hardcoded (cadenas magicas, IPs hardcoded, acceso directo a <code>$_ENV</code>), clases dios y archivos cuello de botella.</p>`,
      zh: `<h2 id="how-aigiscode-parses-php">AigisCode如何解析PHP</h2>
<p>AigisCode使用 <strong>tree-sitter</strong> 将PHP文件解析为完整的语法树。与基于正则表达式的工具不同，tree-sitter理解PHP的语法——嵌套类、匿名函数、复杂的use语句都能正确解析。</p>

<h2 id="symbol-extraction">符号提取</h2>
<p>对于每个PHP文件，AigisCode提取：类（包括抽象和final类）、带可见性修饰符的函数和方法、导入（<code>use</code>语句）、命名空间、trait和常量/属性。所有符号存储在规范化的SQLite索引中。</p>

<h2 id="framework-support">框架支持</h2>
<p><strong>Laravel</strong> — 插件理解service provider、facade、Eloquent模型、Blade模板和路由注册。<strong>WordPress</strong> — 插件处理基于hook的架构：<code>add_action</code>、<code>add_filter</code>、插件引导文件和shortcode。</p>

<h2 id="detection-capabilities">检测能力</h2>
<p>PHP完全索引后，AigisCode运行检测器：循环依赖（Tarjan算法）、死代码（未使用的导入、未引用的方法、孤立类）、硬编码值（魔法字符串、硬编码IP、直接访问 <code>$_ENV</code>）、上帝类和瓶颈文件。</p>`,
      hi: `<h2 id="how-aigiscode-parses-php">AigisCode PHP को कैसे पार्स करता है</h2>
<p>AigisCode PHP फ़ाइलों को पूर्ण सिंटैक्स ट्री में पार्स करने के लिए <strong>tree-sitter</strong> का उपयोग करता है। regex-आधारित टूल के विपरीत, tree-sitter PHP की ग्रामर को समझता है — नेस्टेड क्लास, एनॉनिमस फ़ंक्शन, जटिल use स्टेटमेंट सभी सही ढंग से पार्स होते हैं।</p>

<h2 id="symbol-extraction">सिंबल एक्सट्रैक्शन</h2>
<p>प्रत्येक PHP फ़ाइल के लिए, AigisCode निकालता है: क्लास (abstract और final सहित), विज़िबिलिटी मॉडिफायर के साथ फ़ंक्शन और मेथड, इम्पोर्ट (<code>use</code> स्टेटमेंट), नेमस्पेस, ट्रेट और कॉन्स्टेंट/प्रॉपर्टी। सभी सिंबल नॉर्मलाइज़्ड SQLite इंडेक्स में स्टोर होते हैं।</p>

<h2 id="framework-support">फ्रेमवर्क सपोर्ट</h2>
<p><strong>Laravel</strong> — प्लगइन service providers, facades, Eloquent मॉडल, Blade टेम्पलेट और रूट रजिस्ट्रेशन को समझता है। <strong>WordPress</strong> — प्लगइन hook-आधारित आर्किटेक्चर को संभालता है: <code>add_action</code>, <code>add_filter</code>, प्लगइन बूटस्ट्रैप फ़ाइलें और shortcodes।</p>

<h2 id="detection-capabilities">डिटेक्शन क्षमताएं</h2>
<p>PHP पूरी तरह इंडेक्स होने पर, AigisCode डिटेक्टर चलाता है: सर्कुलर डिपेंडेंसी (Tarjan एल्गोरिथम), डेड कोड (अप्रयुक्त इम्पोर्ट, अनरेफ़रेंस्ड मेथड, ऑर्फ़न क्लास), हार्डवायर्ड वैल्यू (मैजिक स्ट्रिंग, हार्डकोडेड IP, <code>$_ENV</code> तक सीधी पहुंच), गॉड क्लास और बॉटलनेक फ़ाइलें।</p>`,
      pt: `<h2 id="how-aigiscode-parses-php">Como AigisCode analisa PHP</h2>
<p>AigisCode usa <strong>tree-sitter</strong> para analisar arquivos PHP em uma arvore sintatica completa. Diferente de ferramentas baseadas em regex, tree-sitter entende a gramatica do PHP — classes aninhadas, funcoes anonimas, declaracoes use complexas sao todas analisadas corretamente.</p>

<h2 id="symbol-extraction">Extracao de simbolos</h2>
<p>Para cada arquivo PHP, AigisCode extrai: classes (incluindo abstract e final), funcoes e metodos com modificadores de visibilidade, imports (declaracoes <code>use</code>), namespaces, traits e constantes/propriedades. Todos os simbolos sao armazenados em um indice SQLite normalizado.</p>

<h2 id="framework-support">Suporte a frameworks</h2>
<p><strong>Laravel</strong> — O plugin entende service providers, facades, modelos Eloquent, templates Blade e registro de rotas. <strong>WordPress</strong> — O plugin lida com a arquitetura baseada em hooks: <code>add_action</code>, <code>add_filter</code>, arquivos bootstrap de plugins e shortcodes.</p>

<h2 id="detection-capabilities">Capacidades de deteccao</h2>
<p>Com PHP totalmente indexado, AigisCode executa detectores: dependencias circulares (algoritmo de Tarjan), codigo morto (imports nao utilizados, metodos nao referenciados, classes orfas), valores hardcoded (strings magicas, IPs hardcoded, acesso direto a <code>$_ENV</code>), god classes e arquivos gargalo.</p>`,
      ar: `<h2 id="how-aigiscode-parses-php">كيف يحلل AigisCode شيفرة PHP</h2>
<p>يستخدم AigisCode محلل tree-sitter لاستخراج الهيكل الكامل لملفات PHP: الفئات والدوال وفضاءات الأسماء والسمات وتعليمات use. يبني من ذلك رسماً بيانياً للتبعيات يكشف العلاقات عبر الملفات.</p>

<h2 id="detection-capabilities">إمكانيات الاكتشاف</h2>
<ul>
<li><strong>التبعيات الدائرية</strong> — دورات بين فئات PHP عبر تعليمات use وإنشاء الفئات.</li>
<li><strong>الشيفرة الميتة</strong> — فئات وطرق ودوال غير مرجعية.</li>
<li><strong>القيم الثابتة</strong> — سلاسل سحرية وعناوين IP مشفّرة ووصول <code>$_ENV</code> خارج التكوين.</li>
<li><strong>انتهاكات فضاء الأسماء</strong> — استيرادات تكسر حدود فضاء الأسماء المحددة.</li>
</ul>

<h2 id="framework-support">دعم الأطر</h2>
<p>تحسّن إضافات Laravel وWordPress التحليل بوعي بمزوّدي الخدمات ونماذج Eloquent وخطافات WordPress وقوالب Blade. تقلل الإيجابيات الكاذبة من الأنماط الاتفاقية.</p>

<h2 id="symbol-extraction">استخراج الرموز</h2>
<p>لكل ملف PHP يستخرج AigisCode: الفئات والواجهات والسمات وفضاءات الأسماء وتعليمات use والطرق (عامة وخاصة ومحمية) والخصائص والثوابت.</p>`,
      pl: `<h2 id="how-aigiscode-parses-php">Jak AigisCode parsuje PHP</h2>
<p>AigisCode uzywa <strong>tree-sitter</strong> do parsowania PHP. Wyodrebnia klasy, funkcje, importy, przestrzenie nazw, traity i stale.</p>
<h2 id="framework-support">Obsluga frameworkow</h2>
<p><strong>Laravel</strong> — Rozumie service providery, fasady, Eloquent, Blade i routy. <strong>WordPress</strong> — Obsluguje hooki, filtry, bootstrap i shortcody.</p>
<h2 id="detection-capabilities">Mozliwosci detekcji</h2>
<p>Cykliczne zaleznosci, martwy kod, zakodowane wartosci, god klasy i waskie gardla.</p>`,
    },
    relatedSlugs: ['python', 'typescript', 'javascript'],
  },

  /* ---- 2. Python ---- */
  {
    slug: 'python',
    icon: 'FilePy',
    title: {
      en: 'Python Analysis',
      cs: 'Analýza Pythonu',
      fr: 'Analyse Python',
      es: 'Análisis de Python',
      zh: 'Python 分析',
      hi: 'Python विश्लेषण',
      pt: 'Análise de Python',
      ar: 'تحليل Python',
      pl: 'Analiza Python',
      bn: 'Python বিশ্লেষণ',
    },
    shortDescription: {
      en: 'Full structural analysis of Python codebases using Python\'s native AST module. Tracks imports, classes, functions, decorators, and type hints to build an accurate dependency graph and detect dead code, cycles, and hardwired values.',
      cs: 'Kompletní strukturální analýza Python kódových bází pomocí nativního AST modulu Pythonu. Sleduje importy, třídy, funkce, dekorátory a typové anotace pro sestavení přesného grafu závislostí a detekci mrtvého kódu, cyklů a natvrdo zapsaných hodnot.',
      fr: 'Analyse structurelle complète des bases de code Python avec le module AST natif de Python. Suit les imports, classes, fonctions, décorateurs et annotations de types pour construire un graphe de dépendances précis et détecter le code mort, les cycles et les valeurs codées en dur.',
      es: 'Análisis estructural completo de bases de código Python utilizando el módulo AST nativo de Python. Rastrea imports, clases, funciones, decoradores y anotaciones de tipos para construir un grafo de dependencias preciso y detectar código muerto, ciclos y valores codificados.',
      zh: '使用 Python 原生 AST 模块对 Python 代码库进行完整的结构分析。跟踪 import、类、函数、装饰器和类型注解，构建精确的依赖图并检测死代码、循环依赖和硬编码值。',
      hi: 'Python के नेटिव AST मॉड्यूल का उपयोग करके Python कोडबेस का संपूर्ण संरचनात्मक विश्लेषण। सटीक निर्भरता ग्राफ बनाने और मृत कोड, चक्र और हार्डकोड किए गए मानों का पता लगाने के लिए import, कक्षाओं, फ़ंक्शन, डेकोरेटर और टाइप हिंट को ट्रैक करता है।',
      pt: 'Análise estrutural completa de bases de código Python usando o módulo AST nativo do Python. Rastreia imports, classes, funções, decoradores e anotações de tipos para construir um grafo de dependências preciso e detectar código morto, ciclos e valores codificados.',
      ar: 'تحليل هيكلي كامل لقواعد شيفرة Python باستخدام وحدة AST الأصلية في Python. يتتبع الاستيرادات والفئات والدوال والمزخرفات وتلميحات الأنواع لبناء رسم بياني دقيق للتبعيات واكتشاف الشيفرة الميتة والدورات والقيم الثابتة.',
      pl: 'Pelna analiza strukturalna baz kodu Python z uzyciem natywnego modulu AST Pythona. Sledzi importy, hierarchie klas, wywolania funkcji i przypisania na poziomie modulow, aby zbudowac kompletny graf zaleznosci.',
      bn: 'Python-এর নেটিভ AST মডিউল ব্যবহার করে Python কোডবেসের সম্পূর্ণ কাঠামোগত বিশ্লেষণ। সঠিক ডিপেন্ডেন্সি গ্রাফ তৈরি করতে এবং ডেড কোড, চক্র এবং হার্ডওয়্যার্ড ভ্যালু শনাক্ত করতে ইমপোর্ট, ক্লাস, ফাংশন, ডেকোরেটর এবং টাইপ হিন্ট ট্র্যাক করে।',
    },
    metaDescription: {
      en: 'Analyze Python codebases for circular imports, dead code, hardwired values, and architectural issues. AigisCode uses tree-sitter-python to parse classes, functions, decorators, and type hints.',
      cs: 'Analyzujte Python kódové báze na cyklické importy, mrtvý kód, natvrdo zapsané hodnoty a architektonické problémy. AigisCode používá tree-sitter-python k parsování tříd, funkcí, dekorátorů a typových anotací.',
      fr: 'Analysez les bases de code Python pour détecter les imports circulaires, le code mort, les valeurs codées en dur et les problèmes architecturaux. AigisCode utilise tree-sitter-python pour parser les classes, fonctions, décorateurs et annotations de types.',
      es: 'Analice bases de código Python en busca de imports circulares, código muerto, valores codificados y problemas arquitectónicos. AigisCode utiliza tree-sitter-python para analizar clases, funciones, decoradores y anotaciones de tipos.',
      zh: '分析 Python 代码库中的循环 import、死代码、硬编码值和架构问题。AigisCode 使用 tree-sitter-python 解析类、函数、装饰器和类型注解。',
      hi: 'चक्रीय import, मृत कोड, हार्डकोड किए गए मान और आर्किटेक्चर संबंधी समस्याओं के लिए Python कोडबेस का विश्लेषण करें। AigisCode कक्षाओं, फ़ंक्शन, डेकोरेटर और टाइप हिंट को पार्स करने के लिए tree-sitter-python का उपयोग करता है।',
      pt: 'Analise bases de código Python para imports circulares, código morto, valores codificados e problemas arquiteturais. AigisCode usa tree-sitter-python para analisar classes, funções, decoradores e anotações de tipos.',
      ar: 'تحليل قواعد شيفرة Python للاستيرادات الدائرية والشيفرة الميتة والقيم الثابتة والمشكلات المعمارية. يستخدم AigisCode محلل tree-sitter-python لتحليل الفئات والدوال والمزخرفات وتلميحات الأنواع.',
      pl: 'Analizuj bazy kodu Python pod katem cyklicznych importow, martwego kodu, zakodowanych wartosci i problemow architektonicznych. AigisCode uzywa parsowania tree-sitter-python z pelnym rozwiazywaniem importow.',
      bn: 'সার্কুলার ইমপোর্ট, ডেড কোড, হার্ডওয়্যার্ড ভ্যালু এবং আর্কিটেকচারাল সমস্যার জন্য Python কোডবেস বিশ্লেষণ করুন। AigisCode ক্লাস, ফাংশন, ডেকোরেটর এবং টাইপ হিন্ট পার্স করতে tree-sitter-python ব্যবহার করে।',
    },
    parser: 'tree-sitter-python',
    extensions: '.py',
    features: [
      'Class and method extraction',
      'Function declarations',
      'Import and from-import statements',
      'Decorator detection',
      'Type hint parsing',
      'Module-level variable tracking',
      '__all__ export analysis',
      'Relative import resolution',
    ],
    detectorSupport: {
      deadCode: true,
      hardwiring: true,
      graphAnalysis: true,
    },
    frameworkPlugins: ['Django'],
    content: {
      en: `<h2 id="how-aigiscode-parses-python">How AigisCode Parses Python</h2>
<p>AigisCode uses Python's built-in <strong>AST module</strong> to parse Python source files. The AST module is the same parser that CPython itself uses, so it handles every valid Python syntax construct — walrus operators, match statements, f-strings, type parameter syntax, and all other modern Python features are parsed correctly.</p>
<p>Unlike tree-sitter (which AigisCode uses for PHP, TypeScript, JavaScript, and Vue), the tree-sitter-python module provides perfect fidelity for Python code because it is the reference parser. This means zero false negatives from parse failures and no grammar version lag.</p>

<h2 id="symbol-extraction">Symbol Extraction</h2>
<p>For each Python file, AigisCode extracts:</p>
<ul>
<li><strong>Classes</strong> — Class definitions including base classes, metaclasses, and dataclass decorators. Inheritance edges are tracked as outbound dependencies.</li>
<li><strong>Functions</strong> — Top-level functions and methods. Nested functions and lambdas are tracked for completeness but do not create new dependency edges.</li>
<li><strong>Imports</strong> — Both <code>import module</code> and <code>from module import name</code> forms. Relative imports (<code>from . import sibling</code>) are resolved against the package structure. Star imports (<code>from module import *</code>) are flagged separately.</li>
<li><strong>Decorators</strong> — Decorator usage is captured to identify framework-registered functions (e.g., <code>@app.route</code>, <code>@pytest.fixture</code>) that should not be flagged as dead code.</li>
<li><strong>Type hints</strong> — Type annotations on function parameters, return types, and variable annotations are parsed. Forward references in string annotations are resolved where possible.</li>
<li><strong>Module-level variables</strong> — Constants and configuration variables defined at module scope are tracked for hardwiring analysis.</li>
<li><strong>__all__ exports</strong> — When a module defines <code>__all__</code>, AigisCode uses it to determine which names are part of the public API vs. internal implementation details.</li>
</ul>

<h2 id="framework-support">Framework Support</h2>
<h3 id="django-plugin">Django</h3>
<p>The Django plugin understands Django's conventions:</p>
<ul>
<li><strong>Models</strong> — Django model classes and their field definitions are recognized. ForeignKey and ManyToMany string references are not flagged as hardwired values.</li>
<li><strong>Views</strong> — Function-based views and class-based views referenced in <code>urls.py</code> are treated as entry points.</li>
<li><strong>Management commands</strong> — Classes inheriting from <code>BaseCommand</code> in <code>management/commands/</code> are recognized as entry points.</li>
<li><strong>Signals</strong> — Functions connected via <code>@receiver</code> decorators or <code>signal.connect()</code> are not flagged as dead code.</li>
<li><strong>Template tags</strong> — Functions decorated with <code>@register.filter</code>, <code>@register.simple_tag</code>, etc. are recognized as used code.</li>
<li><strong>Settings</strong> — The <code>settings.py</code> module and its <code>INSTALLED_APPS</code>, <code>MIDDLEWARE</code>, and other configuration lists are parsed to discover additional entry points.</li>
</ul>

<h2 id="detection-capabilities">Detection Capabilities</h2>
<p>Python-specific detection highlights:</p>
<ul>
<li><strong>Circular imports</strong> — Python's circular import behavior is notoriously subtle. AigisCode detects both static cycles (import at module level) and deferred cycles (import inside functions). The analyzer marks function-level imports as lower-severity runtime cycles since they are a common and intentional pattern in Python.</li>
<li><strong>Dead code</strong> — Unused imports are detected with awareness of <code>__all__</code>, <code>__init__.py</code> re-exports, and type-checking-only imports (guarded by <code>if TYPE_CHECKING:</code>). Unreferenced functions, classes, and methods are flagged with confidence levels that account for dynamic dispatch patterns.</li>
<li><strong>Hardwired values</strong> — Environment variable access via <code>os.environ</code>, <code>os.getenv()</code>, or <code>environ.get()</code> outside designated config modules is flagged. Repeated string literals, magic numbers, and hardcoded URLs are detected across the codebase.</li>
<li><strong>God classes</strong> — Classes with too many methods or attributes are identified, which is especially useful in Django projects where model classes can grow unchecked.</li>
</ul>

<h2 id="example-findings">Example Findings</h2>
<pre><code># Analyze a Django project
aigiscode analyze /path/to/django-project

# Sample findings:
#
# Circular import: app/services/auth.py
#                 → app/services/permissions.py
#                 → app/models/user.py
#                 → app/services/auth.py
#
# Dead code:   app/utils/legacy_helpers.py:format_phone()
#              0 call sites, confidence: high
#
# Hardwiring:  app/views/payment.py:87
#              os.environ["STRIPE_SECRET_KEY"] outside config
#              → Move to settings.py or django-environ</code></pre>`,
      cs: `<h2 id="how-aigiscode-parses-python">Jak AigisCode parsuje Python</h2>
<p>AigisCode používá vestavěný <strong>AST modul</strong> Pythonu k parsování zdrojových souborů. AST modul je stejný parser, který používá samotný CPython, takže zpracovává každou platnou syntaktickou konstrukci Pythonu — walrus operátory, match příkazy, f-stringy a moderní type parametry.</p>

<h2 id="symbol-extraction">Extrakce symbolů</h2>
<p>Pro každý Python soubor AigisCode extrahuje: třídy (včetně dědičnosti a metaclass), funkce, importy (<code>import</code> i <code>from...import</code>), dekorátory, typové anotace, proměnné na úrovni modulu a <code>__all__</code> exporty. Relativní importy jsou řešeny proti struktuře balíčku.</p>

<h2 id="framework-support">Podpora frameworků</h2>
<p><strong>Django</strong> — Plugin rozumí Django modelům, pohledům (FBV i CBV) registrovaným v <code>urls.py</code>, management příkazům, signálům, šablonovým tagům a nastavení <code>INSTALLED_APPS</code>.</p>

<h2 id="detection-capabilities">Detekční schopnosti</h2>
<p>Python specifické detekce: cyklické importy (statické i odložené), mrtvý kód s ohledem na <code>__all__</code> a <code>TYPE_CHECKING</code> bloky, hardwired hodnoty (<code>os.environ</code> mimo konfiguraci), god třídy a bottleneck soubory.</p>`,
      fr: `<h2 id="how-aigiscode-parses-python">Comment AigisCode analyse Python</h2>
<p>AigisCode utilise le <strong>module AST</strong> integre de Python pour parser les fichiers sources. Le module AST est le meme parseur que CPython utilise, gerant toute la syntaxe Python valide — operateur walrus, match, f-strings et parametres de type modernes.</p>

<h2 id="symbol-extraction">Extraction de symboles</h2>
<p>Pour chaque fichier Python, AigisCode extrait : classes (avec heritage et metaclass), fonctions, imports (<code>import</code> et <code>from...import</code>), decorateurs, annotations de type, variables au niveau du module et exports <code>__all__</code>. Les imports relatifs sont resolus par rapport a la structure du package.</p>

<h2 id="framework-support">Support des frameworks</h2>
<p><strong>Django</strong> — Le plugin comprend les modeles Django, les vues (FBV et CBV) enregistrees dans <code>urls.py</code>, les commandes de gestion, les signaux, les template tags et les parametres <code>INSTALLED_APPS</code>.</p>

<h2 id="detection-capabilities">Capacites de detection</h2>
<p>Detections specifiques a Python : imports circulaires (statiques et differes), code mort avec conscience de <code>__all__</code> et des blocs <code>TYPE_CHECKING</code>, valeurs codees en dur (<code>os.environ</code> hors configuration), classes dieu et fichiers goulots.</p>`,
      es: `<h2 id="how-aigiscode-parses-python">Como AigisCode analiza Python</h2>
<p>AigisCode usa el <strong>modulo AST</strong> integrado de Python para analizar archivos fuente. El modulo AST es el mismo parser que usa CPython, manejando toda la sintaxis Python valida — operador walrus, match, f-strings y parametros de tipo modernos.</p>

<h2 id="symbol-extraction">Extraccion de simbolos</h2>
<p>Para cada archivo Python, AigisCode extrae: clases (con herencia y metaclass), funciones, imports (<code>import</code> y <code>from...import</code>), decoradores, anotaciones de tipo, variables a nivel de modulo y exportaciones <code>__all__</code>. Los imports relativos se resuelven contra la estructura del paquete.</p>

<h2 id="framework-support">Soporte de frameworks</h2>
<p><strong>Django</strong> — El plugin entiende modelos Django, vistas (FBV y CBV) registradas en <code>urls.py</code>, comandos de gestion, senales, template tags y configuracion <code>INSTALLED_APPS</code>.</p>

<h2 id="detection-capabilities">Capacidades de deteccion</h2>
<p>Detecciones especificas de Python: imports circulares (estaticos y diferidos), codigo muerto con consciencia de <code>__all__</code> y bloques <code>TYPE_CHECKING</code>, valores hardcoded (<code>os.environ</code> fuera de configuracion), clases dios y archivos cuello de botella.</p>`,
      zh: `<h2 id="how-aigiscode-parses-python">AigisCode如何解析Python</h2>
<p>AigisCode使用Python内置的 <strong>AST模块</strong> 解析源文件。AST模块是CPython本身使用的同一解析器，能处理所有有效的Python语法——海象运算符、match语句、f-string和现代类型参数。</p>

<h2 id="symbol-extraction">符号提取</h2>
<p>对于每个Python文件，AigisCode提取：类（包括继承和元类）、函数、导入（<code>import</code>和<code>from...import</code>）、装饰器、类型注解、模块级变量和<code>__all__</code>导出。相对导入根据包结构解析。</p>

<h2 id="framework-support">框架支持</h2>
<p><strong>Django</strong> — 插件理解Django模型、在<code>urls.py</code>中注册的视图（FBV和CBV）、管理命令、信号、模板标签和<code>INSTALLED_APPS</code>设置。</p>

<h2 id="detection-capabilities">检测能力</h2>
<p>Python特有检测：循环导入（静态和延迟）、感知<code>__all__</code>和<code>TYPE_CHECKING</code>块的死代码、硬编码值（配置外的<code>os.environ</code>）、上帝类和瓶颈文件。</p>`,
      hi: `<h2 id="how-aigiscode-parses-python">AigisCode Python को कैसे पार्स करता है</h2>
<p>AigisCode स्रोत फ़ाइलों को पार्स करने के लिए Python के बिल्ट-इन <strong>AST मॉड्यूल</strong> का उपयोग करता है। AST मॉड्यूल वही पार्सर है जो CPython स्वयं उपयोग करता है, इसलिए यह हर वैध Python सिंटैक्स को हैंडल करता है — walrus ऑपरेटर, match स्टेटमेंट, f-strings और आधुनिक टाइप पैरामीटर।</p>

<h2 id="symbol-extraction">सिंबल एक्सट्रैक्शन</h2>
<p>प्रत्येक Python फ़ाइल के लिए, AigisCode निकालता है: क्लास (इनहेरिटेंस और मेटाक्लास सहित), फ़ंक्शन, इम्पोर्ट (<code>import</code> और <code>from...import</code>), डेकोरेटर, टाइप एनोटेशन, मॉड्यूल-लेवल वेरिएबल और <code>__all__</code> एक्सपोर्ट।</p>

<h2 id="framework-support">फ्रेमवर्क सपोर्ट</h2>
<p><strong>Django</strong> — प्लगइन Django मॉडल, <code>urls.py</code> में रजिस्टर्ड व्यू (FBV और CBV), मैनेजमेंट कमांड, सिग्नल, टेम्पलेट टैग और <code>INSTALLED_APPS</code> सेटिंग को समझता है।</p>

<h2 id="detection-capabilities">डिटेक्शन क्षमताएं</h2>
<p>Python विशिष्ट डिटेक्शन: सर्कुलर इम्पोर्ट (स्टैटिक और डिफर्ड), <code>__all__</code> और <code>TYPE_CHECKING</code> ब्लॉक के प्रति जागरूक डेड कोड, हार्डवायर्ड वैल्यू (कॉन्फ़िग के बाहर <code>os.environ</code>), गॉड क्लास और बॉटलनेक फ़ाइलें।</p>`,
      pt: `<h2 id="how-aigiscode-parses-python">Como AigisCode analisa Python</h2>
<p>AigisCode usa o <strong>modulo AST</strong> integrado do Python para analisar arquivos fonte. O modulo AST e o mesmo parser que o CPython usa, lidando com toda a sintaxe Python valida — operador walrus, match, f-strings e parametros de tipo modernos.</p>

<h2 id="symbol-extraction">Extracao de simbolos</h2>
<p>Para cada arquivo Python, AigisCode extrai: classes (com heranca e metaclass), funcoes, imports (<code>import</code> e <code>from...import</code>), decoradores, anotacoes de tipo, variaveis a nivel de modulo e exportacoes <code>__all__</code>. Imports relativos sao resolvidos contra a estrutura do pacote.</p>

<h2 id="framework-support">Suporte a frameworks</h2>
<p><strong>Django</strong> — O plugin entende modelos Django, views (FBV e CBV) registradas em <code>urls.py</code>, comandos de gerenciamento, sinais, template tags e configuracao <code>INSTALLED_APPS</code>.</p>

<h2 id="detection-capabilities">Capacidades de deteccao</h2>
<p>Deteccoes especificas do Python: imports circulares (estaticos e adiados), codigo morto com consciencia de <code>__all__</code> e blocos <code>TYPE_CHECKING</code>, valores hardcoded (<code>os.environ</code> fora da configuracao), god classes e arquivos gargalo.</p>`,
      ar: `<h2 id="how-aigiscode-parses-python">كيف يحلل AigisCode شيفرة Python</h2>
<p>يستخدم AigisCode وحدة AST الأصلية في Python لتحليل دقيق للشيفرة المصدرية. يتتبع الاستيرادات والفئات والدوال والمزخرفات وتلميحات الأنواع لبناء رسم بياني كامل للتبعيات.</p>

<h2 id="detection-capabilities">إمكانيات الاكتشاف</h2>
<ul>
<li><strong>الاستيرادات الدائرية</strong> — دورات بين وحدات Python عبر تعليمات import وfrom-import.</li>
<li><strong>الشيفرة الميتة</strong> — استيرادات غير مستخدمة وفئات مهجورة وطرق غير مرجعية.</li>
<li><strong>القيم الثابتة</strong> — وصول <code>os.environ</code> خارج التكوين وسلاسل سحرية وقيم حرفية متكررة.</li>
<li><strong>الفئات الضخمة</strong> — فئات بأكثر من ١٥ طريقة واقتران عالٍ.</li>
</ul>

<h2 id="python-specific">تحديات خاصة بـ Python</h2>
<p>يتعامل AigisCode مع البيئات الافتراضية وملفات الترحيل والاستيرادات الديناميكية. يدعم التخطيطات المسطحة والقائمة على الحزم ويتعرف على أنماط إطار Django.</p>

<h2 id="symbol-extraction">استخراج الرموز</h2>
<p>لكل ملف Python يستخرج AigisCode: تعليمات import وfrom-import وتعريفات الفئات والدوال والمزخرفات وتلميحات الأنواع والمتغيرات على مستوى الوحدة.</p>`,
      pl: `<h2 id="how-aigiscode-parses-python">Jak AigisCode parsuje Python</h2>
<p>Natywny modul <strong>ast</strong> Pythona. Zero zewnetrznych zaleznosci.</p>
<h2 id="framework-support">Obsluga frameworkow</h2>
<p><strong>Django</strong> — Modele, widoki, URL, polecenia zarzadzania i sygnaly.</p>
<h2 id="detection-capabilities">Mozliwosci detekcji</h2>
<p>Cykliczne importy, martwy kod, zakodowane wartosci i god klasy.</p>`,
    },
    relatedSlugs: ['php', 'typescript', 'ruby'],
  },

  /* ---- 3. TypeScript ---- */
  {
    slug: 'typescript',
    icon: 'FileTs',
    title: {
      en: 'TypeScript Analysis',
      cs: 'Analýza TypeScriptu',
      fr: 'Analyse TypeScript',
      es: 'Análisis de TypeScript',
      zh: 'TypeScript 分析',
      hi: 'TypeScript विश्लेषण',
      pt: 'Análise de TypeScript',
      ar: 'تحليل TypeScript',
      pl: 'Analiza TypeScript',
      bn: 'TypeScript বিশ্লেষণ',
    },
    shortDescription: {
      en: 'Comprehensive TypeScript analysis using tree-sitter parsing. Extracts classes, functions, interfaces, type aliases, imports, and module structure to detect circular dependencies, dead exports, and hardwired configuration across .ts and .tsx files.',
      cs: 'Komplexní analýza TypeScriptu pomocí tree-sitter parseru. Extrahuje třídy, funkce, rozhraní, typové aliasy, importy a strukturu modulů pro detekci cyklických závislostí, mrtvých exportů a natvrdo zapsané konfigurace v souborech .ts a .tsx.',
      fr: 'Analyse complète de TypeScript avec le parseur tree-sitter. Extrait les classes, fonctions, interfaces, alias de types, imports et la structure des modules pour détecter les dépendances circulaires, les exports morts et la configuration codée en dur dans les fichiers .ts et .tsx.',
      es: 'Análisis completo de TypeScript utilizando el parser tree-sitter. Extrae clases, funciones, interfaces, alias de tipos, imports y estructura de módulos para detectar dependencias circulares, exports muertos y configuración codificada en archivos .ts y .tsx.',
      zh: '使用 tree-sitter 解析器进行全面的 TypeScript 分析。提取类、函数、接口、类型别名、import 和模块结构，检测 .ts 和 .tsx 文件中的循环依赖、死 export 和硬编码配置。',
      hi: 'tree-sitter पार्सर का उपयोग करके व्यापक TypeScript विश्लेषण। .ts और .tsx फ़ाइलों में चक्रीय निर्भरता, मृत export और हार्डकोड किए गए कॉन्फ़िगरेशन का पता लगाने के लिए कक्षाओं, फ़ंक्शन, इंटरफ़ेस, टाइप एलियास, import और मॉड्यूल संरचना को निकालता है।',
      pt: 'Análise abrangente de TypeScript usando o parser tree-sitter. Extrai classes, funções, interfaces, aliases de tipos, imports e estrutura de módulos para detectar dependências circulares, exports mortos e configuração codificada em arquivos .ts e .tsx.',
      ar: 'تحليل شامل لـ TypeScript باستخدام محلل tree-sitter. يستخرج الفئات والدوال والواجهات وأسماء الأنواع المستعارة والاستيرادات وهيكل الوحدات لاكتشاف التبعيات الدائرية والتصديرات الميتة والتكوين الثابت عبر ملفات .ts و.tsx.',
      pl: 'Kompleksowa analiza TypeScript z parsowaniem tree-sitter. Wyodrebnia klasy, funkcje, interfejsy, aliasy typow, enumy i re-eksporty z pelnym rozwiazywaniem aliasow sciezek.',
      bn: 'tree-sitter পার্সিং ব্যবহার করে ব্যাপক TypeScript বিশ্লেষণ। .ts এবং .tsx ফাইল জুড়ে সার্কুলার ডিপেন্ডেন্সি, ডেড এক্সপোর্ট এবং হার্ডওয়্যার্ড কনফিগারেশন শনাক্ত করতে ক্লাস, ফাংশন, ইন্টারফেস, টাইপ অ্যালিয়াস, ইমপোর্ট এবং মডিউল কাঠামো এক্সট্র্যাক্ট করে।',
    },
    metaDescription: {
      en: 'Analyze TypeScript codebases for circular dependencies, dead exports, hardwired values, and structural issues. AigisCode parses .ts and .tsx files with full type-aware symbol extraction.',
      cs: 'Analyzujte TypeScript kódové báze na cyklické závislosti, mrtvé exporty, natvrdo zapsané hodnoty a strukturální problémy. AigisCode parsuje soubory .ts a .tsx s plnou typově orientovanou extrakcí symbolů.',
      fr: 'Analysez les bases de code TypeScript pour détecter les dépendances circulaires, les exports morts, les valeurs codées en dur et les problèmes structurels. AigisCode parse les fichiers .ts et .tsx avec une extraction de symboles tenant compte des types.',
      es: 'Analice bases de código TypeScript en busca de dependencias circulares, exports muertos, valores codificados y problemas estructurales. AigisCode analiza archivos .ts y .tsx con extracción de símbolos con reconocimiento de tipos.',
      zh: '分析 TypeScript 代码库中的循环依赖、死 export、硬编码值和结构问题。AigisCode 解析 .ts 和 .tsx 文件，支持完整的类型感知符号提取。',
      hi: 'चक्रीय निर्भरता, मृत export, हार्डकोड किए गए मान और संरचनात्मक समस्याओं के लिए TypeScript कोडबेस का विश्लेषण करें। AigisCode पूर्ण टाइप-अवेयर सिंबल एक्सट्रैक्शन के साथ .ts और .tsx फ़ाइलों को पार्स करता है।',
      pt: 'Analise bases de código TypeScript para dependências circulares, exports mortos, valores codificados e problemas estruturais. AigisCode analisa arquivos .ts e .tsx com extração de símbolos com reconhecimento de tipos.',
      ar: 'تحليل قواعد شيفرة TypeScript للتبعيات الدائرية والتصديرات الميتة والقيم الثابتة والمشكلات الهيكلية. يحلل AigisCode ملفات .ts و.tsx مع استخراج رموز واعٍ بالأنواع.',
      pl: 'Analizuj bazy kodu TypeScript pod katem cyklicznych zaleznosci, martwych eksportow, zakodowanych wartosci i problemow strukturalnych. AigisCode rozwiazuje aliasy sciezek i pliki deklaracji.',
      bn: 'সার্কুলার ডিপেন্ডেন্সি, ডেড এক্সপোর্ট, হার্ডওয়্যার্ড ভ্যালু এবং কাঠামোগত সমস্যার জন্য TypeScript কোডবেস বিশ্লেষণ করুন। AigisCode সম্পূর্ণ টাইপ-সচেতন সিম্বল এক্সট্রাকশন সহ .ts এবং .tsx ফাইল পার্স করে।',
    },
    parser: 'tree-sitter',
    extensions: '.ts / .tsx',
    features: [
      'Class and method extraction',
      'Function declarations',
      'Import and re-export statements',
      'Interface definitions',
      'Type alias declarations',
      'Enum extraction',
      'Path alias resolution (@/)',
      'Barrel file detection',
    ],
    detectorSupport: {
      deadCode: true,
      hardwiring: true,
      graphAnalysis: true,
    },
    frameworkPlugins: [],
    content: {
      en: `<h2 id="how-aigiscode-parses-typescript">How AigisCode Parses TypeScript</h2>
<p>AigisCode uses <strong>tree-sitter</strong> with the TypeScript grammar to parse <code>.ts</code> and <code>.tsx</code> files into a complete syntax tree. Tree-sitter handles all TypeScript syntax including generics, conditional types, template literal types, satisfies expressions, and JSX/TSX elements.</p>
<p>The parser distinguishes between value-level and type-level imports, which is critical for accurate dead code detection. A type-only import (<code>import type { User } from './models'</code>) creates a dependency edge for graph analysis but is handled differently during dead code evaluation since the TypeScript compiler erases it at build time.</p>

<h2 id="symbol-extraction">Symbol Extraction</h2>
<p>For each TypeScript file, AigisCode extracts:</p>
<ul>
<li><strong>Classes</strong> — Class declarations with full inheritance tracking. Abstract classes, decorators (common in Angular and NestJS), and generic type parameters are captured.</li>
<li><strong>Functions</strong> — Named function declarations, arrow function assignments, and exported function expressions. Overload signatures are collapsed into a single symbol entry.</li>
<li><strong>Imports</strong> — ES module imports including named imports, default imports, namespace imports (<code>import * as</code>), and side-effect imports (<code>import './polyfill'</code>). Re-exports (<code>export { name } from './module'</code>) are tracked as both an import and an export.</li>
<li><strong>Interfaces</strong> — Interface declarations and their extends clauses. Interfaces that extend other interfaces create dependency edges.</li>
<li><strong>Type aliases</strong> — Type alias declarations (<code>type UserID = string</code>) are tracked. Complex type references within aliases create dependency edges to the files that define the referenced types.</li>
<li><strong>Enums</strong> — Both regular and const enums are extracted. Enum member values are checked for hardwired patterns.</li>
</ul>

<h2 id="path-alias-resolution">Path Alias Resolution</h2>
<p>TypeScript projects commonly use path aliases configured in <code>tsconfig.json</code>. AigisCode resolves these aliases using the <code>js_import_aliases</code> policy field:</p>
<pre><code>// Policy configuration:
{
  "graph": {
    "js_import_aliases": {
      "@/": "src/",
      "@components/": "src/components/",
      "@utils/": "src/utils/"
    }
  }
}</code></pre>
<p>This ensures that <code>import { Button } from '@/components/Button'</code> correctly resolves to <code>src/components/Button.ts</code> in the dependency graph, giving you accurate cycle detection and coupling analysis.</p>

<h2 id="detection-capabilities">Detection Capabilities</h2>
<ul>
<li><strong>Circular dependencies</strong> — Cycle detection accounts for barrel files (<code>index.ts</code> re-export files) that can create false cycles. The analyzer identifies when a cycle passes through a barrel file and reports both the full cycle and the simplified version.</li>
<li><strong>Dead exports</strong> — Exported symbols that are never imported by any other file in the project are flagged. This is especially valuable in large TypeScript projects where refactoring leaves behind exported functions that nothing uses.</li>
<li><strong>Hardwired values</strong> — API endpoint strings, hardcoded port numbers, magic string comparisons, and environment variable access via <code>process.env</code> outside config files are detected.</li>
<li><strong>Bottleneck files</strong> — Files with high fan-in (many importers) are identified. In TypeScript projects, utility files and shared type modules often become unintentional bottleneck points that slow down builds and increase blast radius of changes.</li>
</ul>

<h2 id="example-findings">Example Findings</h2>
<pre><code># Analyze a TypeScript monorepo
aigiscode analyze /path/to/ts-project

# Sample findings:
#
# Strong cycle: src/services/AuthService.ts
#              → src/services/UserService.ts
#              → src/repositories/UserRepo.ts
#              → src/services/AuthService.ts
#
# Dead export:  src/utils/formatters.ts:formatCurrency()
#              Exported but never imported, confidence: high
#
# Hardwiring:  src/api/client.ts:12
#              Hardcoded URL "https://api.example.com/v2"
#              → Move to environment config</code></pre>`,
      cs: `<h2 id="how-aigiscode-parses-typescript">Jak AigisCode parsuje TypeScript</h2>
<p>AigisCode používá <strong>tree-sitter</strong> s TypeScript gramatikou k parsování souborů <code>.ts</code> a <code>.tsx</code>. Tree-sitter zpracovává veškerou TypeScript syntaxi včetně generik, podmíněných typů, template literal typů a JSX/TSX elementů.</p>
<p>Parser rozlišuje importy na úrovni hodnot a typů, což je klíčové pro přesnou detekci mrtvého kódu. Type-only import (<code>import type { User } from './models'</code>) vytváří hranu závislosti, ale je zpracován odlišně při vyhodnocování mrtvého kódu.</p>

<h2 id="symbol-extraction">Extrakce symbolů</h2>
<p>Pro každý TypeScript soubor AigisCode extrahuje: třídy, funkce, importy (včetně re-exportů), rozhraní, typové aliasy a výčtové typy. Aliasy cest (jako <code>@/</code>) jsou řešeny pomocí konfigurace <code>js_import_aliases</code> v politice.</p>

<h2 id="detection-capabilities">Detekční schopnosti</h2>
<ul>
<li><strong>Cyklické závislosti</strong> — Detekce cyklů zohledňuje barrel soubory (<code>index.ts</code>), které mohou vytvářet falešné cykly.</li>
<li><strong>Mrtvé exporty</strong> — Exportované symboly, které žádný jiný soubor nikdy neimportuje, jsou označeny.</li>
<li><strong>Hardwired hodnoty</strong> — API endpointy, natvrdo zapsaná čísla portů a přístup k <code>process.env</code> mimo konfigurační soubory.</li>
<li><strong>Soubory úzkých míst</strong> — Soubory s vysokým fan-in, které se stávají neúmyslnými body úzkých míst.</li>
</ul>`,
      fr: `<h2 id="how-aigiscode-parses-typescript">Comment AigisCode analyse TypeScript</h2>
<p>AigisCode utilise <strong>tree-sitter</strong> avec la grammaire TypeScript pour parser les fichiers <code>.ts</code> et <code>.tsx</code>. Tree-sitter gere toute la syntaxe TypeScript y compris les generiques, les types conditionnels, les types de template litteral et les elements JSX/TSX.</p>
<p>Le parseur distingue les imports au niveau des valeurs et des types, ce qui est crucial pour une detection precise du code mort. Un import de type uniquement (<code>import type { User } from './models'</code>) cree une arete de dependance mais est traite differemment pour le code mort.</p>

<h2 id="symbol-extraction">Extraction de symboles</h2>
<p>Pour chaque fichier TypeScript, AigisCode extrait : classes, fonctions, imports (y compris re-exports), interfaces, alias de types et enums. Les alias de chemin (comme <code>@/</code>) sont resolus via <code>js_import_aliases</code> dans la politique.</p>

<h2 id="detection-capabilities">Capacites de detection</h2>
<ul>
<li><strong>Dependances circulaires</strong> — La detection de cycles tient compte des fichiers barrel (<code>index.ts</code>) qui peuvent creer de faux cycles.</li>
<li><strong>Exports morts</strong> — Les symboles exportes jamais importes par un autre fichier sont signales.</li>
<li><strong>Valeurs codees en dur</strong> — Endpoints d'API, numeros de port codes en dur et acces a <code>process.env</code> hors des fichiers de configuration.</li>
<li><strong>Fichiers goulots</strong> — Fichiers avec un fan-in eleve devenant des goulots non intentionnels.</li>
</ul>`,
      es: `<h2 id="how-aigiscode-parses-typescript">Como AigisCode analiza TypeScript</h2>
<p>AigisCode usa <strong>tree-sitter</strong> con la gramatica TypeScript para analizar archivos <code>.ts</code> y <code>.tsx</code>. Tree-sitter maneja toda la sintaxis TypeScript incluyendo genericos, tipos condicionales, tipos de template literal y elementos JSX/TSX.</p>
<p>El parser distingue entre imports a nivel de valor y tipo, lo cual es critico para la deteccion precisa de codigo muerto. Un import solo de tipo (<code>import type { User } from './models'</code>) crea una arista de dependencia pero se maneja diferente para codigo muerto.</p>

<h2 id="symbol-extraction">Extraccion de simbolos</h2>
<p>Para cada archivo TypeScript, AigisCode extrae: clases, funciones, imports (incluyendo re-exports), interfaces, alias de tipos y enums. Los alias de ruta (como <code>@/</code>) se resuelven via <code>js_import_aliases</code> en la politica.</p>

<h2 id="detection-capabilities">Capacidades de deteccion</h2>
<ul>
<li><strong>Dependencias circulares</strong> — La deteccion de ciclos tiene en cuenta archivos barrel (<code>index.ts</code>) que pueden crear falsos ciclos.</li>
<li><strong>Exports muertos</strong> — Simbolos exportados nunca importados por otro archivo son marcados.</li>
<li><strong>Valores hardcoded</strong> — Endpoints de API, numeros de puerto hardcoded y acceso a <code>process.env</code> fuera de archivos de configuracion.</li>
<li><strong>Archivos cuello de botella</strong> — Archivos con alto fan-in que se convierten en cuellos de botella no intencionales.</li>
</ul>`,
      zh: `<h2 id="how-aigiscode-parses-typescript">AigisCode如何解析TypeScript</h2>
<p>AigisCode使用 <strong>tree-sitter</strong> 和TypeScript语法来解析 <code>.ts</code> 和 <code>.tsx</code> 文件。Tree-sitter处理所有TypeScript语法，包括泛型、条件类型、模板字面量类型和JSX/TSX元素。</p>
<p>解析器区分值级别和类型级别的导入，这对于准确的死代码检测至关重要。纯类型导入（<code>import type { User } from './models'</code>）会创建依赖边，但在死代码评估时会不同处理。</p>

<h2 id="symbol-extraction">符号提取</h2>
<p>对每个TypeScript文件，AigisCode提取：类、函数、导入（包括重新导出）、接口、类型别名和枚举。路径别名（如 <code>@/</code>）通过策略中的 <code>js_import_aliases</code> 解析。</p>

<h2 id="detection-capabilities">检测能力</h2>
<ul>
<li><strong>循环依赖</strong> — 循环检测考虑到barrel文件（<code>index.ts</code>）可能创建假循环。</li>
<li><strong>死导出</strong> — 从未被其他文件导入的导出符号被标记。</li>
<li><strong>硬编码值</strong> — API端点、硬编码端口号和配置文件外的 <code>process.env</code> 访问。</li>
<li><strong>瓶颈文件</strong> — 高扇入文件成为非预期的瓶颈点。</li>
</ul>`,
      hi: `<h2 id="how-aigiscode-parses-typescript">AigisCode TypeScript को कैसे पार्स करता है</h2>
<p>AigisCode <code>.ts</code> और <code>.tsx</code> फ़ाइलों को पार्स करने के लिए TypeScript ग्रामर के साथ <strong>tree-sitter</strong> का उपयोग करता है। Tree-sitter सभी TypeScript सिंटैक्स को हैंडल करता है जिसमें जेनेरिक्स, कंडीशनल टाइप्स, टेम्पलेट लिटरल टाइप्स और JSX/TSX एलिमेंट्स शामिल हैं।</p>
<p>पार्सर वैल्यू-लेवल और टाइप-लेवल इम्पोर्ट में अंतर करता है, जो सटीक डेड कोड डिटेक्शन के लिए महत्वपूर्ण है।</p>

<h2 id="symbol-extraction">सिंबल एक्सट्रैक्शन</h2>
<p>प्रत्येक TypeScript फ़ाइल के लिए, AigisCode निकालता है: क्लास, फ़ंक्शन, इम्पोर्ट (री-एक्सपोर्ट सहित), इंटरफ़ेस, टाइप एलियास और enum। पाथ एलियास (जैसे <code>@/</code>) पॉलिसी में <code>js_import_aliases</code> के माध्यम से हल किए जाते हैं।</p>

<h2 id="detection-capabilities">डिटेक्शन क्षमताएं</h2>
<ul>
<li><strong>सर्कुलर डिपेंडेंसी</strong> — साइकल डिटेक्शन barrel फ़ाइलों (<code>index.ts</code>) को ध्यान में रखता है जो फ़ॉल्स साइकल बना सकती हैं।</li>
<li><strong>डेड एक्सपोर्ट</strong> — एक्सपोर्टेड सिंबल जो किसी अन्य फ़ाइल द्वारा कभी इम्पोर्ट नहीं किए गए, फ़्लैग किए जाते हैं।</li>
<li><strong>हार्डवायर्ड वैल्यू</strong> — API एंडपॉइंट, हार्डकोडेड पोर्ट नंबर और कॉन्फ़िग फ़ाइलों के बाहर <code>process.env</code> एक्सेस।</li>
<li><strong>बॉटलनेक फ़ाइलें</strong> — उच्च फैन-इन वाली फ़ाइलें जो अनजाने बॉटलनेक पॉइंट बन जाती हैं।</li>
</ul>`,
      pt: `<h2 id="how-aigiscode-parses-typescript">Como AigisCode analisa TypeScript</h2>
<p>AigisCode usa <strong>tree-sitter</strong> com a gramatica TypeScript para analisar arquivos <code>.ts</code> e <code>.tsx</code>. Tree-sitter lida com toda a sintaxe TypeScript incluindo generics, tipos condicionais, tipos de template literal e elementos JSX/TSX.</p>
<p>O parser distingue entre imports a nivel de valor e tipo, o que e critico para deteccao precisa de codigo morto. Um import somente de tipo (<code>import type { User } from './models'</code>) cria uma aresta de dependencia mas e tratado diferentemente para codigo morto.</p>

<h2 id="symbol-extraction">Extracao de simbolos</h2>
<p>Para cada arquivo TypeScript, AigisCode extrai: classes, funcoes, imports (incluindo re-exports), interfaces, aliases de tipos e enums. Aliases de caminho (como <code>@/</code>) sao resolvidos via <code>js_import_aliases</code> na politica.</p>

<h2 id="detection-capabilities">Capacidades de deteccao</h2>
<ul>
<li><strong>Dependencias circulares</strong> — A deteccao de ciclos considera arquivos barrel (<code>index.ts</code>) que podem criar falsos ciclos.</li>
<li><strong>Exports mortos</strong> — Simbolos exportados nunca importados por outro arquivo sao sinalizados.</li>
<li><strong>Valores hardcoded</strong> — Endpoints de API, numeros de porta hardcoded e acesso a <code>process.env</code> fora de arquivos de configuracao.</li>
<li><strong>Arquivos gargalo</strong> — Arquivos com alto fan-in que se tornam gargalos nao intencionais.</li>
</ul>`,
      ar: `<h2 id="how-aigiscode-parses-typescript">كيف يحلل AigisCode شيفرة TypeScript</h2>
<p>يستخدم AigisCode محلل tree-sitter لاستخراج شامل من ملفات .ts و.tsx. يفهم الفئات والدوال والواجهات وأسماء الأنواع المستعارة والاستيرادات وهيكل الوحدات.</p>

<h2 id="monorepo-support">دعم المستودعات الأحادية</h2>
<p>يحل AigisCode أسماء المسارات المستعارة (مثل <code>@/</code> إلى <code>src/</code>) ويتتبع التبعيات عبر الحزم ويحلل ملفات barrel (ملفات <code>index.ts</code> التي تعيد التصدير). يجد المشكلات المعمارية عبر مئات الحزم.</p>

<h2 id="detection-capabilities">إمكانيات الاكتشاف</h2>
<ul>
<li><strong>التبعيات الدائرية</strong> — دورات في تعليمات import بما فيها استيرادات الأنواع.</li>
<li><strong>التصديرات الميتة</strong> — رموز مصدّرة لا يستوردها أي ملف آخر.</li>
<li><strong>القيم الثابتة</strong> — وصول <code>process.env</code> خارج التكوين وعناوين URLs المشفّرة.</li>
<li><strong>مشكلات ملفات Barrel</strong> — إعادة تصديرات مفرطة تخلق تبعيات مخفية.</li>
</ul>

<h2 id="symbol-extraction">استخراج الرموز</h2>
<p>لكل ملف TypeScript يستخرج AigisCode: تعليمات import/export والفئات والواجهات وأسماء الأنواع المستعارة والدوال والتعدادات وتعريفات المتغيرات.</p>`,
      pl: `<h2 id="how-aigiscode-parses-typescript">Jak AigisCode parsuje TypeScript</h2>
<p><strong>tree-sitter</strong> z obsluga interfejsow, aliasow typow, enumow i plikow deklaracji. Aliasy sciezek z <code>tsconfig.json</code> rozwiazywane automatycznie.</p>
<h2 id="detection-capabilities">Mozliwosci detekcji</h2>
<p>Cykliczne zaleznosci, martwy kod, zakodowane wartosci i naruszenia warstw.</p>`,
    },
    relatedSlugs: ['javascript', 'vue', 'python'],
  },

  /* ---- 4. JavaScript ---- */
  {
    slug: 'javascript',
    icon: 'FileJs',
    title: {
      en: 'JavaScript Analysis',
      cs: 'Analýza JavaScriptu',
      fr: 'Analyse JavaScript',
      es: 'Análisis de JavaScript',
      zh: 'JavaScript 分析',
      hi: 'JavaScript विश्लेषण',
      pt: 'Análise de JavaScript',
      ar: 'تحليل JavaScript',
      pl: 'Analiza JavaScript',
      bn: 'JavaScript বিশ্লেষণ',
    },
    shortDescription: {
      en: 'Analyze JavaScript codebases with tree-sitter parsing that understands both ES module imports and CommonJS require() calls. Detects circular dependencies, dead code, and hardwired values across .js, .jsx, and .mjs files.',
      cs: 'Analyzujte JavaScript kódové báze pomocí tree-sitter parseru, který rozumí jak ES module importům, tak CommonJS require() voláním. Detekuje cyklické závislosti, mrtvý kód a natvrdo zapsané hodnoty v souborech .js, .jsx a .mjs.',
      fr: 'Analysez les bases de code JavaScript avec le parseur tree-sitter qui comprend les imports ES modules et les appels CommonJS require(). Détecte les dépendances circulaires, le code mort et les valeurs codées en dur dans les fichiers .js, .jsx et .mjs.',
      es: 'Analice bases de código JavaScript con el parser tree-sitter que comprende tanto los imports de ES modules como las llamadas CommonJS require(). Detecta dependencias circulares, código muerto y valores codificados en archivos .js, .jsx y .mjs.',
      zh: '使用 tree-sitter 解析器分析 JavaScript 代码库，同时理解 ES module import 和 CommonJS require() 调用。检测 .js、.jsx 和 .mjs 文件中的循环依赖、死代码和硬编码值。',
      hi: 'tree-sitter पार्सर के साथ JavaScript कोडबेस का विश्लेषण करें जो ES module import और CommonJS require() कॉल दोनों को समझता है। .js, .jsx और .mjs फ़ाइलों में चक्रीय निर्भरता, मृत कोड और हार्डकोड किए गए मानों का पता लगाता है।',
      pt: 'Analise bases de código JavaScript com o parser tree-sitter que compreende tanto imports de ES modules quanto chamadas CommonJS require(). Detecta dependências circulares, código morto e valores codificados em arquivos .js, .jsx e .mjs.',
      ar: 'تحليل قواعد شيفرة JavaScript بمحلل tree-sitter الذي يفهم استيراد وحدات ES واستدعاءات require() في CommonJS. يكتشف التبعيات الدائرية والشيفرة الميتة والقيم الثابتة عبر ملفات .js و.jsx و.mjs.',
      pl: 'Analizuj bazy kodu JavaScript z parsowaniem tree-sitter, ktore rozumie zarowno importy modulow ES, jak i wywolania CommonJS require(). Pelna ekstrakcja symboli z wykrywaniem martwego kodu i cyklicznych zaleznosci.',
      bn: 'ES মডিউল ইমপোর্ট এবং CommonJS require() কল উভয় বোঝে এমন tree-sitter পার্সিং দিয়ে JavaScript কোডবেস বিশ্লেষণ করুন। .js, .jsx এবং .mjs ফাইল জুড়ে সার্কুলার ডিপেন্ডেন্সি, ডেড কোড এবং হার্ডওয়্যার্ড ভ্যালু শনাক্ত করে।',
    },
    metaDescription: {
      en: 'Analyze JavaScript projects for circular dependencies, dead code, and hardwired values. AigisCode parses ES modules and CommonJS with tree-sitter across .js, .jsx, and .mjs files.',
      cs: 'Analyzujte JavaScript projekty na cyklické závislosti, mrtvý kód a natvrdo zapsané hodnoty. AigisCode parsuje ES modules a CommonJS pomocí tree-sitter v souborech .js, .jsx a .mjs.',
      fr: 'Analysez les projets JavaScript pour détecter les dépendances circulaires, le code mort et les valeurs codées en dur. AigisCode parse les ES modules et CommonJS avec tree-sitter dans les fichiers .js, .jsx et .mjs.',
      es: 'Analice proyectos JavaScript en busca de dependencias circulares, código muerto y valores codificados. AigisCode analiza ES modules y CommonJS con tree-sitter en archivos .js, .jsx y .mjs.',
      zh: '分析 JavaScript 项目中的循环依赖、死代码和硬编码值。AigisCode 使用 tree-sitter 在 .js、.jsx 和 .mjs 文件中解析 ES modules 和 CommonJS。',
      hi: 'चक्रीय निर्भरता, मृत कोड और हार्डकोड किए गए मानों के लिए JavaScript प्रोजेक्ट का विश्लेषण करें। AigisCode .js, .jsx और .mjs फ़ाइलों में tree-sitter के साथ ES modules और CommonJS को पार्स करता है।',
      pt: 'Analise projetos JavaScript para dependências circulares, código morto e valores codificados. AigisCode analisa ES modules e CommonJS com tree-sitter em arquivos .js, .jsx e .mjs.',
      ar: 'تحليل مشاريع JavaScript للتبعيات الدائرية والشيفرة الميتة والقيم الثابتة. يحلل AigisCode وحدات ES وCommonJS بـ tree-sitter عبر ملفات .js و.jsx و.mjs.',
      pl: 'Analizuj projekty JavaScript pod katem cyklicznych zaleznosci, martwego kodu i zakodowanych wartosci. AigisCode parsuje importy ESM, require CommonJS i importy dynamiczne.',
      bn: 'সার্কুলার ডিপেন্ডেন্সি, ডেড কোড এবং হার্ডওয়্যার্ড ভ্যালুর জন্য JavaScript প্রজেক্ট বিশ্লেষণ করুন। AigisCode .js, .jsx এবং .mjs ফাইল জুড়ে tree-sitter দিয়ে ES মডিউল এবং CommonJS পার্স করে।',
    },
    parser: 'tree-sitter',
    extensions: '.js / .jsx / .mjs',
    features: [
      'Class and method extraction',
      'Function declarations',
      'ES module imports',
      'CommonJS require() tracking',
      'Dynamic import() detection',
      'Module.exports analysis',
      'JSX component extraction',
      'Named and default export tracking',
    ],
    detectorSupport: {
      deadCode: true,
      hardwiring: true,
      graphAnalysis: true,
    },
    frameworkPlugins: [],
    content: {
      en: `<h2 id="how-aigiscode-parses-javascript">How AigisCode Parses JavaScript</h2>
<p>AigisCode uses <strong>tree-sitter</strong> with the JavaScript grammar to parse <code>.js</code>, <code>.jsx</code>, and <code>.mjs</code> files. The parser handles all modern JavaScript syntax including optional chaining, nullish coalescing, private class fields, top-level await, and JSX.</p>
<p>A key challenge with JavaScript analysis is the coexistence of two module systems. AigisCode handles both:</p>
<ul>
<li><strong>ES Modules</strong> — <code>import</code>/<code>export</code> statements following the ESM specification.</li>
<li><strong>CommonJS</strong> — <code>require()</code> calls and <code>module.exports</code> assignments used in Node.js code.</li>
</ul>
<p>Both module systems are normalized into a unified dependency model, so cycle detection and dead code analysis work consistently regardless of which import style your project uses.</p>

<h2 id="symbol-extraction">Symbol Extraction</h2>
<p>For each JavaScript file, AigisCode extracts:</p>
<ul>
<li><strong>Classes</strong> — ES6+ class declarations and class expressions. Prototype-based "classes" are not tracked as class symbols but their method assignments are captured as function declarations.</li>
<li><strong>Functions</strong> — Named function declarations, arrow function variable assignments, and exported function expressions. Generator functions and async functions are tracked with their modifier.</li>
<li><strong>ES module imports</strong> — Named imports, default imports, namespace imports, and side-effect imports. Dynamic <code>import()</code> expressions are captured as runtime dependency edges.</li>
<li><strong>CommonJS require()</strong> — All <code>require()</code> calls with string literal arguments are tracked. Conditional requires (inside if-blocks or try-catch) are marked as runtime dependencies.</li>
<li><strong>Exports</strong> — Named exports, default exports, and <code>module.exports</code> assignments. Re-exports are tracked as both import and export edges.</li>
<li><strong>JSX components</strong> — Component usage in JSX is tracked as a dependency reference to the component's definition file.</li>
</ul>

<h2 id="mixed-module-handling">Mixed Module Handling</h2>
<p>Many real-world JavaScript projects mix ES modules and CommonJS, especially during migration. AigisCode handles this cleanly:</p>
<pre><code>// ES Module style — tracked as static dependency
import { validateEmail } from './validators';

// CommonJS style — tracked as static dependency
const { sendEmail } = require('./mailer');

// Dynamic import — tracked as runtime dependency
const analytics = await import('./analytics');

// Conditional require — tracked as runtime dependency
if (process.env.NODE_ENV !== 'production') {
  require('./dev-tools');
}</code></pre>
<p>All four patterns above create edges in the dependency graph. Static dependencies (ES imports and top-level requires) feed into strong cycle detection. Runtime dependencies (dynamic imports and conditional requires) feed into runtime cycle detection, which is reported at lower severity.</p>

<h2 id="detection-capabilities">Detection Capabilities</h2>
<ul>
<li><strong>Circular dependencies</strong> — Cycles are detected across both module systems. A cycle that crosses the ESM/CommonJS boundary (e.g., an ES module importing a CommonJS module that requires the ES module) is correctly identified.</li>
<li><strong>Dead code</strong> — Exported functions and classes that are never imported by any file in the project are flagged. CommonJS exports via <code>module.exports</code> are matched against <code>require()</code> destructuring patterns across the codebase.</li>
<li><strong>Hardwired values</strong> — Hardcoded API endpoints, magic strings in comparison expressions, repeated string literals, and direct <code>process.env</code> access outside config modules are detected.</li>
<li><strong>Orphan files</strong> — JavaScript files with no inbound or outbound dependencies are identified. These are often leftover files from past features that were never cleaned up.</li>
</ul>

<h2 id="example-findings">Example Findings</h2>
<pre><code># Analyze a Node.js project
aigiscode analyze /path/to/express-app

# Sample findings:
#
# Strong cycle: src/routes/api.js
#              → src/middleware/auth.js
#              → src/models/Session.js
#              → src/routes/api.js
#
# Dead code:   src/helpers/csv-export.js
#              module.exports used by 0 require() calls
#
# Hardwiring:  src/config/database.js:15
#              Hardcoded "mongodb://localhost:27017/myapp"
#              → Use process.env in config module only</code></pre>`,
      cs: `<h2 id="how-aigiscode-parses-javascript">Jak AigisCode parsuje JavaScript</h2>
<p>AigisCode používá <strong>tree-sitter</strong> s JavaScript gramatikou k parsování souborů <code>.js</code>, <code>.jsx</code> a <code>.mjs</code>. Klíčovou výzvou je koexistence dvou modulových systémů — ES Modules a CommonJS. AigisCode zpracovává oba a normalizuje je do jednotného modelu závislostí.</p>

<h2 id="symbol-extraction">Extrakce symbolů</h2>
<p>Pro každý JavaScript soubor AigisCode extrahuje: třídy (ES6+), funkce, ES module importy, CommonJS <code>require()</code> volání, exporty, dynamické <code>import()</code> výrazy a JSX komponenty. Statické závislosti slouží k detekci silných cyklů, runtime závislosti k detekci běhových cyklů.</p>

<h2 id="detection-capabilities">Detekční schopnosti</h2>
<ul>
<li><strong>Cyklické závislosti</strong> — Cykly jsou detekovány napříč oběma modulovými systémy, včetně cyklů překračujících hranici ESM/CommonJS.</li>
<li><strong>Mrtvý kód</strong> — Exportované funkce a třídy, které žádný soubor neimportuje, jsou označeny.</li>
<li><strong>Hardwired hodnoty</strong> — Natvrdo zapsané API endpointy, magické řetězce, opakované literály a přístup k <code>process.env</code> mimo konfigurační moduly.</li>
<li><strong>Osiřelé soubory</strong> — JavaScript soubory bez příchozích ani odchozích závislostí.</li>
</ul>`,
      fr: `<h2 id="how-aigiscode-parses-javascript">Comment AigisCode analyse JavaScript</h2>
<p>AigisCode utilise <strong>tree-sitter</strong> avec la grammaire JavaScript pour parser les fichiers <code>.js</code>, <code>.jsx</code> et <code>.mjs</code>. Le defi cle est la coexistence de deux systemes de modules — ES Modules et CommonJS. AigisCode gere les deux et les normalise en un modele de dependances unifie.</p>

<h2 id="symbol-extraction">Extraction de symboles</h2>
<p>Pour chaque fichier JavaScript, AigisCode extrait : classes (ES6+), fonctions, imports ES modules, appels CommonJS <code>require()</code>, exports, expressions <code>import()</code> dynamiques et composants JSX. Les dependances statiques servent a la detection de cycles forts, les dependances runtime a la detection de cycles d'execution.</p>

<h2 id="detection-capabilities">Capacites de detection</h2>
<ul>
<li><strong>Dependances circulaires</strong> — Les cycles sont detectes dans les deux systemes de modules, y compris les cycles traversant la frontiere ESM/CommonJS.</li>
<li><strong>Code mort</strong> — Les fonctions et classes exportees jamais importees sont signalees.</li>
<li><strong>Valeurs codees en dur</strong> — Endpoints d'API codes en dur, chaines magiques, litteraux repetes et acces a <code>process.env</code> hors des modules de configuration.</li>
<li><strong>Fichiers orphelins</strong> — Fichiers JavaScript sans dependances entrantes ni sortantes.</li>
</ul>`,
      es: `<h2 id="how-aigiscode-parses-javascript">Como AigisCode analiza JavaScript</h2>
<p>AigisCode usa <strong>tree-sitter</strong> con la gramatica JavaScript para analizar archivos <code>.js</code>, <code>.jsx</code> y <code>.mjs</code>. El desafio clave es la coexistencia de dos sistemas de modulos — ES Modules y CommonJS. AigisCode maneja ambos y los normaliza en un modelo de dependencias unificado.</p>

<h2 id="symbol-extraction">Extraccion de simbolos</h2>
<p>Para cada archivo JavaScript, AigisCode extrae: clases (ES6+), funciones, imports ES modules, llamadas CommonJS <code>require()</code>, exports, expresiones <code>import()</code> dinamicas y componentes JSX. Las dependencias estaticas sirven para la deteccion de ciclos fuertes, las de runtime para ciclos de ejecucion.</p>

<h2 id="detection-capabilities">Capacidades de deteccion</h2>
<ul>
<li><strong>Dependencias circulares</strong> — Los ciclos se detectan en ambos sistemas de modulos, incluyendo ciclos que cruzan la frontera ESM/CommonJS.</li>
<li><strong>Codigo muerto</strong> — Funciones y clases exportadas nunca importadas son marcadas.</li>
<li><strong>Valores hardcoded</strong> — Endpoints de API hardcoded, cadenas magicas, literales repetidos y acceso a <code>process.env</code> fuera de modulos de configuracion.</li>
<li><strong>Archivos huerfanos</strong> — Archivos JavaScript sin dependencias entrantes ni salientes.</li>
</ul>`,
      zh: `<h2 id="how-aigiscode-parses-javascript">AigisCode如何解析JavaScript</h2>
<p>AigisCode使用 <strong>tree-sitter</strong> 和JavaScript语法解析 <code>.js</code>、<code>.jsx</code> 和 <code>.mjs</code> 文件。关键挑战是两种模块系统的共存——ES Modules和CommonJS。AigisCode同时处理两者，将它们规范化为统一的依赖模型。</p>

<h2 id="symbol-extraction">符号提取</h2>
<p>对每个JavaScript文件，AigisCode提取：类（ES6+）、函数、ES模块导入、CommonJS <code>require()</code> 调用、导出、动态 <code>import()</code> 表达式和JSX组件。静态依赖用于强循环检测，运行时依赖用于运行时循环检测。</p>

<h2 id="detection-capabilities">检测能力</h2>
<ul>
<li><strong>循环依赖</strong> — 在两种模块系统中检测循环，包括跨越ESM/CommonJS边界的循环。</li>
<li><strong>死代码</strong> — 从未被导入的导出函数和类被标记。</li>
<li><strong>硬编码值</strong> — 硬编码的API端点、魔法字符串、重复字面量和配置模块外的 <code>process.env</code> 访问。</li>
<li><strong>孤立文件</strong> — 没有入站或出站依赖的JavaScript文件。</li>
</ul>`,
      hi: `<h2 id="how-aigiscode-parses-javascript">AigisCode JavaScript को कैसे पार्स करता है</h2>
<p>AigisCode <code>.js</code>, <code>.jsx</code> और <code>.mjs</code> फ़ाइलों को पार्स करने के लिए JavaScript ग्रामर के साथ <strong>tree-sitter</strong> का उपयोग करता है। मुख्य चुनौती दो मॉड्यूल सिस्टम का सह-अस्तित्व है — ES Modules और CommonJS। AigisCode दोनों को हैंडल करता है और उन्हें एकीकृत डिपेंडेंसी मॉडल में नॉर्मलाइज़ करता है।</p>

<h2 id="symbol-extraction">सिंबल एक्सट्रैक्शन</h2>
<p>प्रत्येक JavaScript फ़ाइल के लिए, AigisCode निकालता है: क्लास (ES6+), फ़ंक्शन, ES मॉड्यूल इम्पोर्ट, CommonJS <code>require()</code> कॉल, एक्सपोर्ट, डायनामिक <code>import()</code> एक्सप्रेशन और JSX कंपोनेंट।</p>

<h2 id="detection-capabilities">डिटेक्शन क्षमताएं</h2>
<ul>
<li><strong>सर्कुलर डिपेंडेंसी</strong> — दोनों मॉड्यूल सिस्टम में साइकल का पता लगाया जाता है, ESM/CommonJS सीमा पार करने वाले साइकल सहित।</li>
<li><strong>डेड कोड</strong> — कभी इम्पोर्ट नहीं किए गए एक्सपोर्टेड फ़ंक्शन और क्लास फ़्लैग किए जाते हैं।</li>
<li><strong>हार्डवायर्ड वैल्यू</strong> — हार्डकोडेड API एंडपॉइंट, मैजिक स्ट्रिंग, दोहराए गए लिटरल और कॉन्फ़िग मॉड्यूल के बाहर <code>process.env</code> एक्सेस।</li>
<li><strong>ऑर्फ़न फ़ाइलें</strong> — बिना इनबाउंड या आउटबाउंड डिपेंडेंसी वाली JavaScript फ़ाइलें।</li>
</ul>`,
      pt: `<h2 id="how-aigiscode-parses-javascript">Como AigisCode analisa JavaScript</h2>
<p>AigisCode usa <strong>tree-sitter</strong> com a gramatica JavaScript para analisar arquivos <code>.js</code>, <code>.jsx</code> e <code>.mjs</code>. O desafio chave e a coexistencia de dois sistemas de modulos — ES Modules e CommonJS. AigisCode lida com ambos e os normaliza em um modelo de dependencias unificado.</p>

<h2 id="symbol-extraction">Extracao de simbolos</h2>
<p>Para cada arquivo JavaScript, AigisCode extrai: classes (ES6+), funcoes, imports ES modules, chamadas CommonJS <code>require()</code>, exports, expressoes <code>import()</code> dinamicas e componentes JSX. Dependencias estaticas servem para deteccao de ciclos fortes, dependencias de runtime para ciclos de execucao.</p>

<h2 id="detection-capabilities">Capacidades de deteccao</h2>
<ul>
<li><strong>Dependencias circulares</strong> — Ciclos sao detectados em ambos os sistemas de modulos, incluindo ciclos que cruzam a fronteira ESM/CommonJS.</li>
<li><strong>Codigo morto</strong> — Funcoes e classes exportadas nunca importadas sao sinalizadas.</li>
<li><strong>Valores hardcoded</strong> — Endpoints de API hardcoded, strings magicas, literais repetidos e acesso a <code>process.env</code> fora de modulos de configuracao.</li>
<li><strong>Arquivos orfaos</strong> — Arquivos JavaScript sem dependencias de entrada ou saida.</li>
</ul>`,
      ar: `<h2 id="how-aigiscode-parses-javascript">كيف يحلل AigisCode شيفرة JavaScript</h2>
<p>يحلل AigisCode ملفات JavaScript بمحلل tree-sitter الذي يفهم وحدات ES (import/export) واستدعاءات CommonJS (require/module.exports). يعالج ملفات .js و.jsx و.mjs.</p>

<h2 id="mixed-modules">معالجة الوحدات المختلطة</h2>
<p>المشاريع الحقيقية غالباً تخلط بين أنماط الوحدات. يتتبع AigisCode كلاً من تعليمات import لـ ESM واستدعاءات <code>require()</code> لـ CommonJS ويبني رسماً بيانياً موحداً للتبعيات.</p>

<h2 id="detection-capabilities">إمكانيات الاكتشاف</h2>
<ul>
<li><strong>التبعيات الدائرية</strong> — دورات في مزيج من استيرادات ESM واستدعاءات require.</li>
<li><strong>الشيفرة الميتة</strong> — تصديرات لا يستوردها أحد ودوال غير مرجعية.</li>
<li><strong>القيم الثابتة</strong> — وصول <code>process.env</code> خارج التكوين وعناوين URLs وIPs المشفّرة.</li>
<li><strong>تحليل JSX</strong> — يفهم مراجع المكونات في صياغة JSX.</li>
</ul>

<h2 id="symbol-extraction">استخراج الرموز</h2>
<p>لكل ملف JavaScript يستخرج AigisCode: استيرادات/تصديرات ES وaستدعاءات require/module.exports والفئات والدوال وتعبيرات الدوال السهمية.</p>`,
      pl: `<h2 id="how-aigiscode-parses-javascript">Jak AigisCode parsuje JavaScript</h2>
<p><strong>tree-sitter</strong> z obsluga ESM i CommonJS <code>require()</code>. Importy dynamiczne rowniez sledzone.</p>
<h2 id="detection-capabilities">Mozliwosci detekcji</h2>
<p>Cykliczne zaleznosci, martwy kod, zakodowane wartosci i analiza architektury.</p>`,
    },
    relatedSlugs: ['typescript', 'vue', 'php'],
  },

  /* ---- 5. Vue ---- */
  {
    slug: 'vue',
    icon: 'FileVue',
    title: {
      en: 'Vue Analysis',
      cs: 'Analýza Vue',
      fr: 'Analyse Vue',
      es: 'Análisis de Vue',
      zh: 'Vue 分析',
      hi: 'Vue विश्लेषण',
      pt: 'Análise de Vue',
      ar: 'تحليل Vue',
      pl: 'Analiza Vue',
      bn: 'Vue বিশ্লেষণ',
    },
    shortDescription: {
      en: 'Single File Component parsing for Vue.js projects using tree-sitter. Analyzes <script setup>, composition API, composables, template refs, and component imports to build a complete dependency graph of your Vue application.',
      cs: 'Parsování Single File Components pro Vue.js projekty pomocí tree-sitter. Analyzuje <script setup>, composition API, composables, template refs a importy komponent pro sestavení kompletního grafu závislostí vaší Vue aplikace.',
      fr: 'Analyse des Single File Components pour les projets Vue.js avec tree-sitter. Analyse <script setup>, l\'API de composition, les composables, les refs de template et les imports de composants pour construire un graphe de dépendances complet de votre application Vue.',
      es: 'Análisis de Single File Components para proyectos Vue.js utilizando tree-sitter. Analiza <script setup>, la API de composición, composables, refs de template e imports de componentes para construir un grafo de dependencias completo de su aplicación Vue.',
      zh: '使用 tree-sitter 为 Vue.js 项目解析单文件组件。分析 <script setup>、组合式 API、composables、模板 ref 和组件 import，构建 Vue 应用程序的完整依赖图。',
      hi: 'tree-sitter का उपयोग करके Vue.js प्रोजेक्ट के लिए Single File Component पार्सिंग। आपके Vue एप्लिकेशन का संपूर्ण निर्भरता ग्राफ बनाने के लिए <script setup>, composition API, composables, template ref और कंपोनेंट import का विश्लेषण करता है।',
      pt: 'Análise de Single File Components para projetos Vue.js usando tree-sitter. Analisa <script setup>, API de composição, composables, refs de template e imports de componentes para construir um grafo de dependências completo da sua aplicação Vue.',
      ar: 'تحليل مكونات الملف الواحد لمشاريع Vue.js باستخدام tree-sitter. يحلل <script setup> وcomposition API والعناصر التركيبية ومراجع القوالب واستيراد المكونات لبناء رسم بياني كامل لتبعيات تطبيق Vue.',
      pl: 'Parsowanie komponentow jednoplikowych dla projektow Vue.js z tree-sitter. Analizuje <script setup>, composable API kompozycji, referencje szablonow i zaleznosci stylow.',
      bn: 'tree-sitter ব্যবহার করে Vue.js প্রজেক্টের জন্য Single File Component পার্সিং। আপনার Vue অ্যাপ্লিকেশনের সম্পূর্ণ ডিপেন্ডেন্সি গ্রাফ তৈরি করতে <script setup>, composition API, composable, টেমপ্লেট ref এবং কম্পোনেন্ট ইমপোর্ট বিশ্লেষণ করে।',
    },
    metaDescription: {
      en: 'Analyze Vue.js projects for circular dependencies, dead composables, and hardwired values. AigisCode parses .vue Single File Components with full script setup and composition API support.',
      cs: 'Analyzujte Vue.js projekty na cyklické závislosti, mrtvé composables a natvrdo zapsané hodnoty. AigisCode parsuje .vue Single File Components s plnou podporou script setup a composition API.',
      fr: 'Analysez les projets Vue.js pour détecter les dépendances circulaires, les composables morts et les valeurs codées en dur. AigisCode parse les Single File Components .vue avec support complet de script setup et de l\'API de composition.',
      es: 'Analice proyectos Vue.js en busca de dependencias circulares, composables muertos y valores codificados. AigisCode analiza Single File Components .vue con soporte completo de script setup y la API de composición.',
      zh: '分析 Vue.js 项目中的循环依赖、死 composables 和硬编码值。AigisCode 解析 .vue 单文件组件，完整支持 script setup 和组合式 API。',
      hi: 'चक्रीय निर्भरता, मृत composables और हार्डकोड किए गए मानों के लिए Vue.js प्रोजेक्ट का विश्लेषण करें। AigisCode पूर्ण script setup और composition API सपोर्ट के साथ .vue Single File Components को पार्स करता है।',
      pt: 'Analise projetos Vue.js para dependências circulares, composables mortos e valores codificados. AigisCode analisa Single File Components .vue com suporte completo a script setup e API de composição.',
      ar: 'تحليل مشاريع Vue.js للتبعيات الدائرية والعناصر التركيبية الميتة والقيم الثابتة. يحلل AigisCode مكونات الملف الواحد .vue مع دعم كامل لإعداد script وواجهة composition API.',
      pl: 'Analizuj projekty Vue.js pod katem cyklicznych zaleznosci, martwych composable i zakodowanych wartosci. AigisCode parsuje komponenty jednoplikowe z ekstrakcja skryptow, szablonow i stylow.',
      bn: 'সার্কুলার ডিপেন্ডেন্সি, ডেড composable এবং হার্ডওয়্যার্ড ভ্যালুর জন্য Vue.js প্রজেক্ট বিশ্লেষণ করুন। AigisCode সম্পূর্ণ script setup এবং composition API সাপোর্ট সহ .vue Single File Component পার্স করে।',
    },
    parser: 'tree-sitter',
    extensions: '.vue',
    features: [
      'Single File Component parsing',
      'Script setup support',
      'Composition API tracking',
      'Composable usage detection',
      'Template ref extraction',
      'Component import resolution',
      'defineProps/defineEmits extraction',
      'Auto-import awareness',
    ],
    detectorSupport: {
      deadCode: true,
      hardwiring: true,
      graphAnalysis: true,
    },
    frameworkPlugins: [],
    content: {
      en: `<h2 id="how-aigiscode-parses-vue">How AigisCode Parses Vue</h2>
<p>AigisCode uses <strong>tree-sitter</strong> to parse <code>.vue</code> Single File Components (SFCs). Vue's SFC format combines a <code>&lt;template&gt;</code>, <code>&lt;script&gt;</code>, and <code>&lt;style&gt;</code> block in a single file. AigisCode focuses on the script block, parsing it as TypeScript or JavaScript depending on the <code>lang</code> attribute.</p>
<p>Both Vue 3's <code>&lt;script setup&gt;</code> syntax and the traditional Options API / Composition API patterns are supported. The parser understands that <code>&lt;script setup&gt;</code> blocks have implicit returns — every top-level binding is automatically available in the template — which affects how dead code analysis works for Vue components.</p>

<h2 id="symbol-extraction">Symbol Extraction</h2>
<p>For each <code>.vue</code> file, AigisCode extracts:</p>
<ul>
<li><strong>Script setup bindings</strong> — All top-level <code>const</code>, <code>let</code>, <code>function</code>, and <code>import</code> declarations in <code>&lt;script setup&gt;</code> blocks. These are the component's public API to the template.</li>
<li><strong>Composable calls</strong> — Calls to composition functions like <code>useRouter()</code>, <code>useStore()</code>, and custom composables are tracked as dependencies. Composable files that are never called are flagged as potential dead code.</li>
<li><strong>Component imports</strong> — Imported components used in <code>&lt;script setup&gt;</code> are automatically registered. AigisCode tracks which components are imported and whether they actually appear in the template or are unused.</li>
<li><strong>defineProps and defineEmits</strong> — Compiler macros that define the component's interface. Prop types and event declarations are extracted for cross-component analysis.</li>
<li><strong>Template refs</strong> — <code>ref()</code> calls that correspond to template <code>ref</code> attributes are tracked. Refs defined in script but never bound in the template may indicate dead code.</li>
<li><strong>Provide/inject pairs</strong> — <code>provide()</code> and <code>inject()</code> calls are tracked to detect orphaned providers or consumers.</li>
</ul>

<h2 id="vue-specific-challenges">Vue-Specific Analysis Challenges</h2>
<p>Vue components present unique challenges for static analysis:</p>
<ul>
<li><strong>Implicit template usage</strong> — In <code>&lt;script setup&gt;</code>, every import and binding is potentially used by the template. AigisCode does not flag these as dead code without confirming they are absent from the template block.</li>
<li><strong>Auto-imports</strong> — Many Vue projects use <code>unplugin-auto-import</code> or similar tools that make Vue APIs (<code>ref</code>, <code>computed</code>, <code>watch</code>) available without explicit imports. AigisCode's policy system lets you exclude these from dead-import analysis.</li>
<li><strong>Dynamic component resolution</strong> — Components loaded via <code>&lt;component :is="..."&gt;</code> or async components cannot be resolved statically. AigisCode reports the static dependency graph and notes where dynamic resolution limits coverage.</li>
</ul>

<h2 id="detection-capabilities">Detection Capabilities</h2>
<ul>
<li><strong>Circular dependencies</strong> — Component import cycles are detected. A common pattern in Vue apps is mutual component references (component A imports component B, which imports component A for nested rendering). AigisCode flags these and suggests using async components or restructuring the component tree.</li>
<li><strong>Dead composables</strong> — Composable files in <code>composables/</code> or <code>hooks/</code> directories that are never imported by any component or other composable are flagged.</li>
<li><strong>Hardwired values</strong> — API URLs, magic strings, and configuration values embedded directly in components instead of pulled from a centralized config or environment are detected.</li>
<li><strong>Orphan components</strong> — Components that exist in the project but are never imported or referenced by any route or parent component.</li>
</ul>

<h2 id="example-findings">Example Findings</h2>
<pre><code># Analyze a Vue 3 project
aigiscode analyze /path/to/vue-app

# Sample findings:
#
# Strong cycle: src/components/TreeNode.vue
#              → src/components/TreeView.vue
#              → src/components/TreeNode.vue
#
# Dead code:   src/composables/useNotifications.ts
#              Exported composable, 0 import sites
#
# Hardwiring:  src/views/Dashboard.vue:34
#              Hardcoded API base "https://api.internal.co/v1"
#              → Move to VITE_API_BASE in .env</code></pre>`,
      cs: `<h2 id="how-aigiscode-parses-vue">Jak AigisCode parsuje Vue</h2>
<p>AigisCode používá <strong>tree-sitter</strong> k parsování <code>.vue</code> Single File Components (SFC). Vue SFC formát kombinuje <code>&lt;template&gt;</code>, <code>&lt;script&gt;</code> a <code>&lt;style&gt;</code> blok v jednom souboru. AigisCode se zaměřuje na script blok a parsuje jej jako TypeScript nebo JavaScript.</p>
<p>Jsou podporovány jak Vue 3 <code>&lt;script setup&gt;</code> syntaxe, tak tradiční Options API / Composition API vzory.</p>

<h2 id="symbol-extraction">Extrakce symbolů</h2>
<p>Pro každý <code>.vue</code> soubor AigisCode extrahuje: script setup bindings, volání composables, importy komponent, <code>defineProps</code> a <code>defineEmits</code> makra, template refs a provide/inject páry.</p>

<h2 id="detection-capabilities">Detekční schopnosti</h2>
<ul>
<li><strong>Cyklické závislosti</strong> — Cykly importů komponent jsou detekovány. Vzájemné reference komponent jsou označeny s návrhem použít async komponenty.</li>
<li><strong>Mrtvé composables</strong> — Soubory composables, které žádná komponenta neimportuje.</li>
<li><strong>Hardwired hodnoty</strong> — API URL a konfigurační hodnoty vložené přímo do komponent.</li>
<li><strong>Osiřelé komponenty</strong> — Komponenty, které žádná routa ani rodičovská komponenta neimportuje.</li>
</ul>`,
      fr: `<h2 id="how-aigiscode-parses-vue">Comment AigisCode analyse Vue</h2>
<p>AigisCode utilise <strong>tree-sitter</strong> pour parser les Single File Components <code>.vue</code> (SFC). Le format SFC Vue combine un bloc <code>&lt;template&gt;</code>, <code>&lt;script&gt;</code> et <code>&lt;style&gt;</code> dans un seul fichier. AigisCode se concentre sur le bloc script et le parse comme TypeScript ou JavaScript.</p>
<p>La syntaxe Vue 3 <code>&lt;script setup&gt;</code> et les patterns traditionnels Options API / Composition API sont supportes.</p>

<h2 id="symbol-extraction">Extraction de symboles</h2>
<p>Pour chaque fichier <code>.vue</code>, AigisCode extrait : bindings script setup, appels de composables, imports de composants, macros <code>defineProps</code> et <code>defineEmits</code>, refs de template et paires provide/inject.</p>

<h2 id="detection-capabilities">Capacites de detection</h2>
<ul>
<li><strong>Dependances circulaires</strong> — Les cycles d'import de composants sont detectes. Les references mutuelles de composants sont signalees avec suggestion d'utiliser des composants async.</li>
<li><strong>Composables morts</strong> — Fichiers de composables jamais importes par aucun composant.</li>
<li><strong>Valeurs codees en dur</strong> — URLs d'API et valeurs de configuration incorporees directement dans les composants.</li>
<li><strong>Composants orphelins</strong> — Composants jamais importes par aucune route ni composant parent.</li>
</ul>`,
      es: `<h2 id="how-aigiscode-parses-vue">Como AigisCode analiza Vue</h2>
<p>AigisCode usa <strong>tree-sitter</strong> para analizar Single File Components <code>.vue</code> (SFC). El formato SFC de Vue combina un bloque <code>&lt;template&gt;</code>, <code>&lt;script&gt;</code> y <code>&lt;style&gt;</code> en un solo archivo. AigisCode se enfoca en el bloque script y lo analiza como TypeScript o JavaScript.</p>
<p>Se soportan tanto la sintaxis Vue 3 <code>&lt;script setup&gt;</code> como los patrones tradicionales Options API / Composition API.</p>

<h2 id="symbol-extraction">Extraccion de simbolos</h2>
<p>Para cada archivo <code>.vue</code>, AigisCode extrae: bindings de script setup, llamadas a composables, imports de componentes, macros <code>defineProps</code> y <code>defineEmits</code>, refs de template y pares provide/inject.</p>

<h2 id="detection-capabilities">Capacidades de deteccion</h2>
<ul>
<li><strong>Dependencias circulares</strong> — Los ciclos de import de componentes se detectan. Las referencias mutuas de componentes se marcan con sugerencia de usar componentes async.</li>
<li><strong>Composables muertos</strong> — Archivos de composables nunca importados por ningun componente.</li>
<li><strong>Valores hardcoded</strong> — URLs de API y valores de configuracion incrustados directamente en componentes.</li>
<li><strong>Componentes huerfanos</strong> — Componentes nunca importados por ninguna ruta ni componente padre.</li>
</ul>`,
      zh: `<h2 id="how-aigiscode-parses-vue">AigisCode如何解析Vue</h2>
<p>AigisCode使用 <strong>tree-sitter</strong> 解析 <code>.vue</code> 单文件组件（SFC）。Vue SFC格式在一个文件中组合 <code>&lt;template&gt;</code>、<code>&lt;script&gt;</code> 和 <code>&lt;style&gt;</code> 块。AigisCode专注于script块，将其解析为TypeScript或JavaScript。</p>
<p>支持Vue 3 <code>&lt;script setup&gt;</code> 语法和传统的Options API / Composition API模式。</p>

<h2 id="symbol-extraction">符号提取</h2>
<p>对每个 <code>.vue</code> 文件，AigisCode提取：script setup绑定、composable调用、组件导入、<code>defineProps</code>和<code>defineEmits</code>宏、模板ref和provide/inject对。</p>

<h2 id="detection-capabilities">检测能力</h2>
<ul>
<li><strong>循环依赖</strong> — 检测组件导入循环。相互引用的组件会被标记并建议使用异步组件。</li>
<li><strong>死composable</strong> — 没有任何组件导入的composable文件。</li>
<li><strong>硬编码值</strong> — 直接嵌入组件中的API URL和配置值。</li>
<li><strong>孤立组件</strong> — 没有任何路由或父组件导入的组件。</li>
</ul>`,
      hi: `<h2 id="how-aigiscode-parses-vue">AigisCode Vue को कैसे पार्स करता है</h2>
<p>AigisCode <code>.vue</code> Single File Components (SFC) को पार्स करने के लिए <strong>tree-sitter</strong> का उपयोग करता है। Vue SFC फॉर्मेट एक ही फ़ाइल में <code>&lt;template&gt;</code>, <code>&lt;script&gt;</code> और <code>&lt;style&gt;</code> ब्लॉक को जोड़ता है। AigisCode स्क्रिप्ट ब्लॉक पर ध्यान केंद्रित करता है।</p>
<p>Vue 3 <code>&lt;script setup&gt;</code> सिंटैक्स और पारंपरिक Options API / Composition API दोनों पैटर्न समर्थित हैं।</p>

<h2 id="symbol-extraction">सिंबल एक्सट्रैक्शन</h2>
<p>प्रत्येक <code>.vue</code> फ़ाइल के लिए, AigisCode निकालता है: script setup बाइंडिंग, composable कॉल, कंपोनेंट इम्पोर्ट, <code>defineProps</code> और <code>defineEmits</code> मैक्रो, टेम्पलेट ref और provide/inject जोड़े।</p>

<h2 id="detection-capabilities">डिटेक्शन क्षमताएं</h2>
<ul>
<li><strong>सर्कुलर डिपेंडेंसी</strong> — कंपोनेंट इम्पोर्ट साइकल का पता लगाया जाता है। म्यूचुअल कंपोनेंट रेफ़रेंस को async कंपोनेंट उपयोग करने के सुझाव के साथ फ़्लैग किया जाता है।</li>
<li><strong>डेड composables</strong> — ऐसी composable फ़ाइलें जो किसी कंपोनेंट द्वारा कभी इम्पोर्ट नहीं की गईं।</li>
<li><strong>हार्डवायर्ड वैल्यू</strong> — सीधे कंपोनेंट में एम्बेड किए गए API URL और कॉन्फ़िगरेशन वैल्यू।</li>
<li><strong>ऑर्फ़न कंपोनेंट</strong> — ऐसे कंपोनेंट जो किसी रूट या पैरेंट कंपोनेंट द्वारा कभी इम्पोर्ट नहीं किए गए।</li>
</ul>`,
      pt: `<h2 id="how-aigiscode-parses-vue">Como AigisCode analisa Vue</h2>
<p>AigisCode usa <strong>tree-sitter</strong> para analisar Single File Components <code>.vue</code> (SFC). O formato SFC do Vue combina um bloco <code>&lt;template&gt;</code>, <code>&lt;script&gt;</code> e <code>&lt;style&gt;</code> em um unico arquivo. AigisCode foca no bloco script e o analisa como TypeScript ou JavaScript.</p>
<p>Tanto a sintaxe Vue 3 <code>&lt;script setup&gt;</code> quanto os padroes tradicionais Options API / Composition API sao suportados.</p>

<h2 id="symbol-extraction">Extracao de simbolos</h2>
<p>Para cada arquivo <code>.vue</code>, AigisCode extrai: bindings script setup, chamadas de composables, imports de componentes, macros <code>defineProps</code> e <code>defineEmits</code>, refs de template e pares provide/inject.</p>

<h2 id="detection-capabilities">Capacidades de deteccao</h2>
<ul>
<li><strong>Dependencias circulares</strong> — Ciclos de import de componentes sao detectados. Referencias mutuas de componentes sao sinalizadas com sugestao de usar componentes async.</li>
<li><strong>Composables mortos</strong> — Arquivos de composables nunca importados por nenhum componente.</li>
<li><strong>Valores hardcoded</strong> — URLs de API e valores de configuracao incorporados diretamente em componentes.</li>
<li><strong>Componentes orfaos</strong> — Componentes nunca importados por nenhuma rota ou componente pai.</li>
</ul>`,
      ar: `<h2 id="how-aigiscode-parses-vue">كيف يحلل AigisCode شيفرة Vue</h2>
<p>يحلل AigisCode مكونات الملف الواحد (.vue) باستخدام tree-sitter. يفهم كتلة <code>&lt;script setup&gt;</code> وcomposition API والعناصر التركيبية ومراجع القوالب واستيراد المكونات.</p>

<h2 id="vue-specific">تحديات خاصة بـ Vue</h2>
<p>تمزج مكونات Vue بين القالب والسكربت والأسلوب في ملف واحد. يستخرج AigisCode التبعيات من كتلة السكربت مع تتبع استخدام المكونات في القالب.</p>

<h2 id="detection-capabilities">إمكانيات الاكتشاف</h2>
<ul>
<li><strong>التبعيات الدائرية</strong> — دورات بين المكونات والعناصر التركيبية.</li>
<li><strong>العناصر التركيبية الميتة</strong> — عناصر تركيبية مصدّرة لا يستخدمها أي مكون.</li>
<li><strong>القيم الثابتة</strong> — وصول <code>import.meta.env</code> خارج التكوين.</li>
<li><strong>المكونات اليتيمة</strong> — مكونات .vue غير مستوردة في أي مكان.</li>
</ul>

<h2 id="symbol-extraction">استخراج الرموز</h2>
<p>لكل ملف .vue يستخرج AigisCode: استيرادات المكونات وتعريفات العناصر التركيبية ومتغيرات ref/reactive والاستيرادات من <code>&lt;script setup&gt;</code> ومراجع المكونات في القالب.</p>`,
      pl: `<h2 id="how-aigiscode-parses-vue">Jak AigisCode parsuje Vue</h2>
<p>Parsowanie komponentow jednoplikowych z tree-sitter. <code>&lt;script setup&gt;</code>, Composition API i Options API.</p>
<h2 id="detection-capabilities">Mozliwosci detekcji</h2>
<p>Cykliczne zaleznosci, martwe composable, zakodowane wartosci.</p>`,
    },
    relatedSlugs: ['typescript', 'javascript', 'php'],
  },

  /* ---- 6. Ruby ---- */
  {
    slug: 'ruby',
    icon: 'Diamond',
    title: {
      en: 'Ruby Analysis',
      cs: 'Analýza Ruby',
      fr: 'Analyse Ruby',
      es: 'Análisis de Ruby',
      zh: 'Ruby 分析',
      hi: 'Ruby विश्लेषण',
      pt: 'Análise de Ruby',
      ar: 'تحليل Ruby',
      pl: 'Analiza Ruby',
      bn: 'Ruby বিশ্লেষণ',
    },
    shortDescription: {
      en: 'Structural analysis of Ruby codebases using tree-sitter parsing. Extracts classes, modules, methods, and require statements to detect circular dependencies, dead methods, and hardwired configuration values across your Ruby project.',
      cs: 'Strukturální analýza Ruby kódových bází pomocí tree-sitter parseru. Extrahuje třídy, moduly, metody a require příkazy pro detekci cyklických závislostí, mrtvých metod a natvrdo zapsaných konfiguračních hodnot v celém Ruby projektu.',
      fr: 'Analyse structurelle des bases de code Ruby avec le parseur tree-sitter. Extrait les classes, modules, méthodes et instructions require pour détecter les dépendances circulaires, les méthodes mortes et les valeurs de configuration codées en dur dans l\'ensemble du projet Ruby.',
      es: 'Análisis estructural de bases de código Ruby utilizando el parser tree-sitter. Extrae clases, módulos, métodos y sentencias require para detectar dependencias circulares, métodos muertos y valores de configuración codificados en todo el proyecto Ruby.',
      zh: '使用 tree-sitter 解析器对 Ruby 代码库进行结构分析。提取类、模块、方法和 require 语句，检测整个 Ruby 项目中的循环依赖、死方法和硬编码配置值。',
      hi: 'tree-sitter पार्सर का उपयोग करके Ruby कोडबेस का संरचनात्मक विश्लेषण। आपके पूरे Ruby प्रोजेक्ट में चक्रीय निर्भरता, मृत विधियों और हार्डकोड किए गए कॉन्फ़िगरेशन मानों का पता लगाने के लिए कक्षाओं, मॉड्यूल, विधियों और require स्टेटमेंट को निकालता है।',
      pt: 'Análise estrutural de bases de código Ruby usando o parser tree-sitter. Extrai classes, módulos, métodos e instruções require para detectar dependências circulares, métodos mortos e valores de configuração codificados em todo o projeto Ruby.',
      ar: 'تحليل هيكلي لقواعد شيفرة Ruby باستخدام محلل tree-sitter. يستخرج الفئات والوحدات والطرق وتعليمات require لاكتشاف التبعيات الدائرية والطرق الميتة وقيم التكوين الثابتة عبر مشروع Ruby.',
      pl: 'Analiza strukturalna baz kodu Ruby z parsowaniem tree-sitter. Wyodrebnia klasy, moduly, metody, stale i instrukcje require/require_relative z wykrywaniem frameworka Rails.',
      bn: 'tree-sitter পার্সিং ব্যবহার করে Ruby কোডবেসের কাঠামোগত বিশ্লেষণ। আপনার Ruby প্রজেক্ট জুড়ে সার্কুলার ডিপেন্ডেন্সি, ডেড মেথড এবং হার্ডওয়্যার্ড কনফিগারেশন ভ্যালু শনাক্ত করতে ক্লাস, মডিউল, মেথড এবং require স্টেটমেন্ট এক্সট্র্যাক্ট করে।',
    },
    metaDescription: {
      en: 'Analyze Ruby codebases for circular dependencies, dead code, hardwired values, and architectural issues. AigisCode uses tree-sitter to parse classes, modules, methods, and require statements.',
      cs: 'Analyzujte Ruby kódové báze na cyklické závislosti, mrtvý kód, natvrdo zapsané hodnoty a architektonické problémy. AigisCode používá tree-sitter k parsování tříd, modulů, metod a require příkazů.',
      fr: 'Analysez les bases de code Ruby pour détecter les dépendances circulaires, le code mort, les valeurs codées en dur et les problèmes architecturaux. AigisCode utilise tree-sitter pour parser les classes, modules, méthodes et instructions require.',
      es: 'Analice bases de código Ruby en busca de dependencias circulares, código muerto, valores codificados y problemas arquitectónicos. AigisCode utiliza tree-sitter para analizar clases, módulos, métodos y sentencias require.',
      zh: '分析 Ruby 代码库中的循环依赖、死代码、硬编码值和架构问题。AigisCode 使用 tree-sitter 解析类、模块、方法和 require 语句。',
      hi: 'चक्रीय निर्भरता, मृत कोड, हार्डकोड किए गए मान और आर्किटेक्चर संबंधी समस्याओं के लिए Ruby कोडबेस का विश्लेषण करें। AigisCode कक्षाओं, मॉड्यूल, विधियों और require स्टेटमेंट को पार्स करने के लिए tree-sitter का उपयोग करता है।',
      pt: 'Analise bases de código Ruby para dependências circulares, código morto, valores codificados e problemas arquiteturais. AigisCode usa tree-sitter para analisar classes, módulos, métodos e instruções require.',
      ar: 'تحليل قواعد شيفرة Ruby للتبعيات الدائرية والشيفرة الميتة والقيم الثابتة والمشكلات المعمارية. يستخدم AigisCode محلل tree-sitter لتحليل الفئات والوحدات والطرق وتعليمات require.',
      pl: 'Analizuj bazy kodu Ruby pod katem cyklicznych zaleznosci, martwego kodu, zakodowanych wartosci i problemow architektonicznych. AigisCode uzywa parsowania tree-sitter z analiza swiadoma Rails.',
      bn: 'সার্কুলার ডিপেন্ডেন্সি, ডেড কোড, হার্ডওয়্যার্ড ভ্যালু এবং আর্কিটেকচারাল সমস্যার জন্য Ruby কোডবেস বিশ্লেষণ করুন। AigisCode ক্লাস, মডিউল, মেথড এবং require স্টেটমেন্ট পার্স করতে tree-sitter ব্যবহার করে।',
    },
    parser: 'tree-sitter',
    extensions: '.rb',
    features: [
      'Class and method extraction',
      'Module and mixin detection',
      'require and require_relative',
      'Block and proc tracking',
      'Constant resolution',
      'Autoload convention support',
      'Gem dependency awareness',
      'DSL method registration',
    ],
    detectorSupport: {
      deadCode: true,
      hardwiring: true,
      graphAnalysis: true,
    },
    frameworkPlugins: ['Rails'],
    content: {
      en: `<h2 id="how-aigiscode-parses-ruby">How AigisCode Parses Ruby</h2>
<p>AigisCode uses <strong>tree-sitter</strong> with the Ruby grammar to parse <code>.rb</code> files into a full syntax tree. Tree-sitter handles Ruby's complex syntax including blocks, procs, heredocs, method_missing patterns, and the various forms of string interpolation and quoting.</p>
<p>Ruby's dynamic nature makes static analysis more challenging than in statically typed languages, but the structural patterns — class hierarchies, module mixins, require chains, and constant references — provide a rich foundation for dependency graph construction and dead code detection.</p>

<h2 id="symbol-extraction">Symbol Extraction</h2>
<p>For each Ruby file, AigisCode extracts:</p>
<ul>
<li><strong>Classes</strong> — Class definitions including superclass references. Nested classes (e.g., <code>Module::ClassName</code>) are tracked with their full qualified name. Reopened classes (defining the same class across multiple files) are linked together.</li>
<li><strong>Modules</strong> — Module definitions and <code>include</code>/<code>extend</code>/<code>prepend</code> statements. Module inclusion creates dependency edges in the graph, similar to trait usage in PHP.</li>
<li><strong>Methods</strong> — Instance methods, class methods (<code>self.method_name</code>), and singleton methods. Visibility modifiers (<code>public</code>, <code>private</code>, <code>protected</code>) are captured for dead code confidence scoring.</li>
<li><strong>Require statements</strong> — Both <code>require</code> and <code>require_relative</code> are tracked. Require paths are resolved relative to the project root and load path. Gem requires (e.g., <code>require 'json'</code>) are identified and excluded from internal dependency analysis.</li>
<li><strong>Constants</strong> — Constant assignments and references are tracked. Constants that reference other classes or modules create dependency edges.</li>
<li><strong>DSL methods</strong> — Common Ruby DSL patterns like <code>attr_accessor</code>, <code>belongs_to</code>, <code>has_many</code>, and callback registrations are recognized as symbol declarations.</li>
</ul>

<h2 id="framework-support">Framework Support</h2>
<h3 id="rails-plugin">Ruby on Rails</h3>
<p>The Rails plugin adapts analysis for Rails conventions:</p>
<ul>
<li><strong>Autoloading</strong> — Rails uses Zeitwerk for autoloading, which means classes are available without explicit <code>require</code> statements. The plugin understands autoload paths and creates dependency edges based on constant references rather than require statements.</li>
<li><strong>ActiveRecord models</strong> — Association declarations (<code>has_many</code>, <code>belongs_to</code>, <code>has_one</code>, <code>has_and_belongs_to_many</code>) create dependency edges between model files. Polymorphic associations are handled through string class references.</li>
<li><strong>Controllers</strong> — Controller classes referenced in <code>config/routes.rb</code> are recognized as entry points and not flagged as dead code.</li>
<li><strong>Concerns</strong> — Modules in <code>app/models/concerns/</code> and <code>app/controllers/concerns/</code> included via <code>include</code> statements are tracked as dependencies.</li>
<li><strong>Callbacks</strong> — Methods registered via <code>before_action</code>, <code>after_create</code>, and other lifecycle hooks are recognized as used even though they have no explicit call sites.</li>
<li><strong>Jobs and mailers</strong> — ActiveJob and ActionMailer subclasses are treated as entry points.</li>
</ul>

<h2 id="detection-capabilities">Detection Capabilities</h2>
<ul>
<li><strong>Circular dependencies</strong> — Require chains and autoload-based constant references are analyzed for cycles. Rails models with circular associations (User has_many Posts, Post belongs_to User) are handled as expected associations rather than architectural cycles.</li>
<li><strong>Dead code</strong> — Methods with no call sites, modules with no include/extend references, and classes with no instantiation or inheritance are flagged. Private methods are flagged with higher confidence since their usage scope is limited to the defining class.</li>
<li><strong>Hardwired values</strong> — Hardcoded database connection strings, API keys in source files, magic strings, and configuration values that should live in <code>config/</code> or environment variables are detected.</li>
<li><strong>God classes</strong> — Models and controllers with excessive numbers of methods are identified. In Rails projects, the "fat model" antipattern often produces classes with 50+ methods that should be decomposed into concerns or service objects.</li>
</ul>

<h2 id="example-findings">Example Findings</h2>
<pre><code># Analyze a Rails project
aigiscode analyze /path/to/rails-app

# Sample findings:
#
# Strong cycle: app/services/billing_service.rb
#              → app/services/subscription_service.rb
#              → app/models/invoice.rb
#              → app/services/billing_service.rb
#
# Dead code:   app/helpers/legacy_format_helper.rb
#              Module included by 0 files, confidence: high
#
# Hardwiring:  app/mailers/notification_mailer.rb:23
#              Hardcoded "smtp.sendgrid.net"
#              → Move to config/environments/*.rb</code></pre>`,
      cs: `<h2 id="how-aigiscode-parses-ruby">Jak AigisCode parsuje Ruby</h2>
<p>AigisCode používá <strong>tree-sitter</strong> s Ruby gramatikou k parsování souborů <code>.rb</code>. Tree-sitter zpracovává komplexní Ruby syntaxi včetně bloků, procs, heredoců a různých forem interpolace řetězců.</p>
<p>Dynamická povaha Ruby činí statickou analýzu náročnější, ale strukturální vzory — hierarchie tříd, modulové mixiny, řetězce require a reference konstant — poskytují základ pro konstrukci grafu závislostí.</p>

<h2 id="symbol-extraction">Extrakce symbolů</h2>
<p>Pro každý Ruby soubor AigisCode extrahuje: třídy (včetně vnořených a znovuotevřených), moduly s <code>include</code>/<code>extend</code>/<code>prepend</code>, metody (instanční, třídní, singleton), <code>require</code> a <code>require_relative</code> příkazy, konstanty a DSL metody.</p>

<h2 id="framework-support">Podpora frameworků</h2>
<p><strong>Ruby on Rails</strong> — Plugin zpracovává Zeitwerk autoloading, ActiveRecord asociace (<code>has_many</code>, <code>belongs_to</code>), kontroléry v <code>config/routes.rb</code>, concerns, callbacky (<code>before_action</code>, <code>after_create</code>) a ActiveJob/ActionMailer podtřídy.</p>

<h2 id="detection-capabilities">Detekční schopnosti</h2>
<ul>
<li><strong>Cyklické závislosti</strong> — Řetězce require a autoload-based reference konstant jsou analyzovány na cykly.</li>
<li><strong>Mrtvý kód</strong> — Metody bez míst volání, moduly bez include/extend referencí a třídy bez instanciace.</li>
<li><strong>Hardwired hodnoty</strong> — Natvrdo zapsané připojovací řetězce k databázi, API klíče a konfigurační hodnoty.</li>
<li><strong>God třídy</strong> — Modely a kontroléry s nadměrným počtem metod, včetně antipattern „fat model".</li>
</ul>`,
      fr: `<h2 id="how-aigiscode-parses-ruby">Comment AigisCode analyse Ruby</h2>
<p>AigisCode utilise <strong>tree-sitter</strong> avec la grammaire Ruby pour parser les fichiers <code>.rb</code>. Tree-sitter gere la syntaxe complexe de Ruby y compris les blocs, procs, heredocs et les diverses formes d'interpolation de chaines.</p>
<p>La nature dynamique de Ruby rend l'analyse statique plus difficile, mais les patterns structurels — hierarchies de classes, mixins de modules, chaines de require et references de constantes — fournissent une base pour la construction du graphe de dependances.</p>

<h2 id="symbol-extraction">Extraction de symboles</h2>
<p>Pour chaque fichier Ruby, AigisCode extrait : classes (imbriquees et reouvertes), modules avec <code>include</code>/<code>extend</code>/<code>prepend</code>, methodes (instance, classe, singleton), instructions <code>require</code> et <code>require_relative</code>, constantes et methodes DSL.</p>

<h2 id="framework-support">Support des frameworks</h2>
<p><strong>Ruby on Rails</strong> — Le plugin gere l'autoloading Zeitwerk, les associations ActiveRecord (<code>has_many</code>, <code>belongs_to</code>), les controleurs dans <code>config/routes.rb</code>, les concerns, les callbacks (<code>before_action</code>, <code>after_create</code>) et les sous-classes ActiveJob/ActionMailer.</p>

<h2 id="detection-capabilities">Capacites de detection</h2>
<ul>
<li><strong>Dependances circulaires</strong> — Les chaines de require et les references de constantes basees sur l'autoload sont analysees pour les cycles.</li>
<li><strong>Code mort</strong> — Methodes sans sites d'appel, modules sans references include/extend et classes sans instanciation.</li>
<li><strong>Valeurs codees en dur</strong> — Chaines de connexion a la base de donnees codees en dur, cles API et valeurs de configuration.</li>
<li><strong>Classes dieu</strong> — Modeles et controleurs avec un nombre excessif de methodes, y compris l'antipattern « fat model ».</li>
</ul>`,
      es: `<h2 id="how-aigiscode-parses-ruby">Como AigisCode analiza Ruby</h2>
<p>AigisCode usa <strong>tree-sitter</strong> con la gramatica Ruby para analizar archivos <code>.rb</code>. Tree-sitter maneja la sintaxis compleja de Ruby incluyendo bloques, procs, heredocs y las diversas formas de interpolacion de cadenas.</p>
<p>La naturaleza dinamica de Ruby hace el analisis estatico mas desafiante, pero los patrones estructurales — jerarquias de clases, mixins de modulos, cadenas de require y referencias de constantes — proporcionan la base para la construccion del grafo.</p>

<h2 id="symbol-extraction">Extraccion de simbolos</h2>
<p>Para cada archivo Ruby, AigisCode extrae: clases (anidadas y reabiertas), modulos con <code>include</code>/<code>extend</code>/<code>prepend</code>, metodos (instancia, clase, singleton), sentencias <code>require</code> y <code>require_relative</code>, constantes y metodos DSL.</p>

<h2 id="framework-support">Soporte de frameworks</h2>
<p><strong>Ruby on Rails</strong> — El plugin maneja el autoloading de Zeitwerk, asociaciones ActiveRecord (<code>has_many</code>, <code>belongs_to</code>), controladores en <code>config/routes.rb</code>, concerns, callbacks (<code>before_action</code>, <code>after_create</code>) y subclases ActiveJob/ActionMailer.</p>

<h2 id="detection-capabilities">Capacidades de deteccion</h2>
<ul>
<li><strong>Dependencias circulares</strong> — Las cadenas de require y las referencias de constantes basadas en autoload se analizan para ciclos.</li>
<li><strong>Codigo muerto</strong> — Metodos sin sitios de llamada, modulos sin referencias include/extend y clases sin instanciacion.</li>
<li><strong>Valores hardcoded</strong> — Cadenas de conexion a base de datos hardcoded, claves API y valores de configuracion.</li>
<li><strong>Clases dios</strong> — Modelos y controladores con numero excesivo de metodos, incluyendo el antipatron "fat model".</li>
</ul>`,
      zh: `<h2 id="how-aigiscode-parses-ruby">AigisCode如何解析Ruby</h2>
<p>AigisCode使用 <strong>tree-sitter</strong> 和Ruby语法解析 <code>.rb</code> 文件。Tree-sitter处理Ruby的复杂语法，包括块、proc、heredoc和各种形式的字符串插值。</p>
<p>Ruby的动态特性使静态分析更具挑战性，但结构模式——类层次、模块mixin、require链和常量引用——为依赖图构建提供了丰富的基础。</p>

<h2 id="symbol-extraction">符号提取</h2>
<p>对每个Ruby文件，AigisCode提取：类（包括嵌套和重新打开的类）、带 <code>include</code>/<code>extend</code>/<code>prepend</code> 的模块、方法（实例、类、单例）、<code>require</code>和<code>require_relative</code>语句、常量和DSL方法。</p>

<h2 id="framework-support">框架支持</h2>
<p><strong>Ruby on Rails</strong> — 插件处理Zeitwerk自动加载、ActiveRecord关联（<code>has_many</code>、<code>belongs_to</code>）、<code>config/routes.rb</code>中的控制器、concerns、回调（<code>before_action</code>、<code>after_create</code>）和ActiveJob/ActionMailer子类。</p>

<h2 id="detection-capabilities">检测能力</h2>
<ul>
<li><strong>循环依赖</strong> — 分析require链和基于自动加载的常量引用中的循环。</li>
<li><strong>死代码</strong> — 无调用点的方法、无include/extend引用的模块和无实例化的类。</li>
<li><strong>硬编码值</strong> — 硬编码的数据库连接字符串、API密钥和配置值。</li>
<li><strong>上帝类</strong> — 方法数过多的模型和控制器，包括"胖模型"反模式。</li>
</ul>`,
      hi: `<h2 id="how-aigiscode-parses-ruby">AigisCode Ruby को कैसे पार्स करता है</h2>
<p>AigisCode <code>.rb</code> फ़ाइलों को पार्स करने के लिए Ruby ग्रामर के साथ <strong>tree-sitter</strong> का उपयोग करता है। Tree-sitter Ruby की जटिल सिंटैक्स को हैंडल करता है जिसमें ब्लॉक, procs, heredocs और स्ट्रिंग इंटरपोलेशन शामिल हैं।</p>
<p>Ruby की डायनामिक प्रकृति स्टैटिक एनालिसिस को अधिक चुनौतीपूर्ण बनाती है, लेकिन संरचनात्मक पैटर्न — क्लास हायरार्की, मॉड्यूल मिक्सिन, require चेन और कॉन्स्टेंट रेफ़रेंस — डिपेंडेंसी ग्राफ निर्माण के लिए आधार प्रदान करते हैं।</p>

<h2 id="symbol-extraction">सिंबल एक्सट्रैक्शन</h2>
<p>प्रत्येक Ruby फ़ाइल के लिए, AigisCode निकालता है: क्लास (नेस्टेड और रीओपन सहित), <code>include</code>/<code>extend</code>/<code>prepend</code> वाले मॉड्यूल, मेथड (इंस्टेंस, क्लास, सिंगलटन), <code>require</code> और <code>require_relative</code> स्टेटमेंट, कॉन्स्टेंट और DSL मेथड।</p>

<h2 id="framework-support">फ्रेमवर्क सपोर्ट</h2>
<p><strong>Ruby on Rails</strong> — प्लगइन Zeitwerk ऑटोलोडिंग, ActiveRecord एसोसिएशन (<code>has_many</code>, <code>belongs_to</code>), <code>config/routes.rb</code> में कंट्रोलर, concerns, कॉलबैक (<code>before_action</code>, <code>after_create</code>) और ActiveJob/ActionMailer सबक्लास को हैंडल करता है।</p>

<h2 id="detection-capabilities">डिटेक्शन क्षमताएं</h2>
<ul>
<li><strong>सर्कुलर डिपेंडेंसी</strong> — require चेन और autoload-आधारित कॉन्स्टेंट रेफ़रेंस का साइकल के लिए विश्लेषण।</li>
<li><strong>डेड कोड</strong> — बिना कॉल साइट वाले मेथड, बिना include/extend रेफ़रेंस वाले मॉड्यूल और बिना इंस्टैंशिएशन वाली क्लास।</li>
<li><strong>हार्डवायर्ड वैल्यू</strong> — हार्डकोडेड डेटाबेस कनेक्शन स्ट्रिंग, API कुंजियां और कॉन्फ़िगरेशन वैल्यू।</li>
<li><strong>गॉड क्लास</strong> — अत्यधिक मेथड वाले मॉडल और कंट्रोलर, "fat model" एंटीपैटर्न सहित।</li>
</ul>`,
      pt: `<h2 id="how-aigiscode-parses-ruby">Como AigisCode analisa Ruby</h2>
<p>AigisCode usa <strong>tree-sitter</strong> com a gramatica Ruby para analisar arquivos <code>.rb</code>. Tree-sitter lida com a sintaxe complexa do Ruby incluindo blocos, procs, heredocs e diversas formas de interpolacao de strings.</p>
<p>A natureza dinamica do Ruby torna a analise estatica mais desafiadora, mas os padroes estruturais — hierarquias de classes, mixins de modulos, cadeias de require e referencias de constantes — fornecem a base para construcao do grafo de dependencias.</p>

<h2 id="symbol-extraction">Extracao de simbolos</h2>
<p>Para cada arquivo Ruby, AigisCode extrai: classes (aninhadas e reabertas), modulos com <code>include</code>/<code>extend</code>/<code>prepend</code>, metodos (instancia, classe, singleton), declaracoes <code>require</code> e <code>require_relative</code>, constantes e metodos DSL.</p>

<h2 id="framework-support">Suporte a frameworks</h2>
<p><strong>Ruby on Rails</strong> — O plugin lida com autoloading Zeitwerk, associacoes ActiveRecord (<code>has_many</code>, <code>belongs_to</code>), controllers em <code>config/routes.rb</code>, concerns, callbacks (<code>before_action</code>, <code>after_create</code>) e subclasses ActiveJob/ActionMailer.</p>

<h2 id="detection-capabilities">Capacidades de deteccao</h2>
<ul>
<li><strong>Dependencias circulares</strong> — Cadeias de require e referencias de constantes baseadas em autoload sao analisadas para ciclos.</li>
<li><strong>Codigo morto</strong> — Metodos sem sites de chamada, modulos sem referencias include/extend e classes sem instanciacao.</li>
<li><strong>Valores hardcoded</strong> — Strings de conexao de banco de dados hardcoded, chaves de API e valores de configuracao.</li>
<li><strong>God classes</strong> — Modelos e controllers com numero excessivo de metodos, incluindo o antipadrao "fat model".</li>
</ul>`,
      ar: `<h2 id="how-aigiscode-parses-ruby">كيف يحلل AigisCode شيفرة Ruby</h2>
<p>يستخدم AigisCode محلل tree-sitter لاستخراج الهيكل من ملفات Ruby. يتتبع الفئات والوحدات والطرق وتعليمات require لبناء رسم بياني كامل للتبعيات.</p>

<h2 id="detection-capabilities">إمكانيات الاكتشاف</h2>
<ul>
<li><strong>التبعيات الدائرية</strong> — دورات بين الفئات عبر تعليمات require وrequire_relative.</li>
<li><strong>الطرق الميتة</strong> — طرق خاصة ومحمية لا تُستدعى أبداً.</li>
<li><strong>القيم الثابتة</strong> — وصول <code>ENV[]</code> خارج التكوين وسلاسل سحرية.</li>
<li><strong>وحدات غير مستخدمة</strong> — وحدات مُضمّنة لكن لا يُشار إليها.</li>
</ul>

<h2 id="rails-support">دعم Ruby on Rails</h2>
<p>يتعرف AigisCode على اتفاقيات Rails مثل النماذج والمتحكمات والترحيلات والمساعدات والمهام. تمنع الاتفاقيات الخاصة بـ Rails الإيجابيات الكاذبة من التحميل التلقائي والاتفاقيات على التكوين.</p>

<h2 id="symbol-extraction">استخراج الرموز</h2>
<p>لكل ملف Ruby يستخرج AigisCode: تعريفات الفئات والوحدات وتعليمات require/require_relative والطرق (عامة وخاصة ومحمية) والثوابت وتضمينات الوحدات.</p>`,
      pl: `<h2 id="how-aigiscode-parses-ruby">Jak AigisCode parsuje Ruby</h2>
<p><strong>tree-sitter</strong> z obsluga klas, modulow, metod i require. Plugin Rails rozumie konwencje.</p>
<h2 id="detection-capabilities">Mozliwosci detekcji</h2>
<p>Cykliczne zaleznosci, martwy kod, zakodowane wartosci i god klasy.</p>`,
    },
    relatedSlugs: ['python', 'php', 'typescript'],
  },
];

/* -------------------------------------------------------------------------- */
/*  Helper Functions                                                          */
/* -------------------------------------------------------------------------- */

export function getLanguageBySlug(slug: string): Language | undefined {
  return languages.find((lang) => lang.slug === slug);
}

export function getRelatedLanguages(slugs: string[]): Language[] {
  return slugs
    .map((slug) => languages.find((lang) => lang.slug === slug))
    .filter((lang): lang is Language => lang !== undefined);
}
