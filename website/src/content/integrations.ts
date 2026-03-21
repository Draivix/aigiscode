/* -------------------------------------------------------------------------- */
/*  Integration Data — AigisCode Marketing Website                          */
/* -------------------------------------------------------------------------- */

export interface Integration {
  slug: string;
  icon: string;
  category: string;
  title: Record<string, string>;
  shortDescription: Record<string, string>;
  metaDescription: Record<string, string>;
  content: Record<string, string>;
  features: string[];
  codeExample?: string;
  status: 'available' | 'coming-soon';
  relatedSlugs: string[];
}

export const integrations: Integration[] = [
  /* ---- 1. CI/CD Pipelines ---- */
  {
    slug: 'ci-cd-pipelines',
    icon: 'FlowArrow',
    category: 'CI/CD',
    status: 'available',
    title: {
      en: 'CI/CD Pipelines',
      cs: 'CI/CD Pipelines',
      fr: 'Pipelines CI/CD',
      es: 'Pipelines CI/CD',
      zh: 'CI/CD 流水线',
      hi: 'CI/CD पाइपलाइन',
      pt: 'Pipelines CI/CD',
      ar: 'خطوط أنابيب CI/CD',
      pl: 'CI/CD Pipelines',
      bn: 'CI/CD Pipeline',
    },
    shortDescription: {
      en: 'Run AigisCode in any CI system to catch architectural issues before they reach production. Supports exit codes, JSON artifacts, and the --skip-ai flag for deterministic analysis.',
      cs: 'Spouštějte AigisCode v libovolném CI systému a odhalte architektonické problémy dříve, než se dostanou do produkce. Podporuje návratové kódy, JSON artefakty a příznak --skip-ai pro deterministickou analýzu.',
      fr: 'Exécutez AigisCode dans n\'importe quel système CI pour détecter les problèmes architecturaux avant qu\'ils n\'atteignent la production. Prend en charge les codes de sortie, les artefacts JSON et le drapeau --skip-ai pour une analyse déterministe.',
      es: 'Ejecute AigisCode en cualquier sistema CI para detectar problemas arquitectónicos antes de que lleguen a producción. Soporta códigos de salida, artefactos JSON y la opción --skip-ai para análisis determinista.',
      zh: '在任意 CI 系统中运行 AigisCode，在架构问题到达生产环境之前将其捕获。支持退出码、JSON 产物以及用于确定性分析的 --skip-ai 标志。',
      hi: 'किसी भी CI सिस्टम में AigisCode चलाएं और आर्किटेक्चरल समस्याओं को प्रोडक्शन तक पहुंचने से पहले पकड़ें। एक्ज़िट कोड, JSON आर्टिफैक्ट्स और डिटरमिनिस्टिक विश्लेषण के लिए --skip-ai फ्लैग का समर्थन करता है।',
      pt: 'Execute o AigisCode em qualquer sistema CI para capturar problemas arquiteturais antes que cheguem à produção. Suporta códigos de saída, artefatos JSON e a flag --skip-ai para análise determinística.',
      ar: 'شغّل AigisCode في أي نظام CI لاكتشاف المشكلات المعمارية قبل وصولها للإنتاج. يدعم أكواد الخروج ومخرجات JSON وعلامة --skip-ai للتحليل الحتمي.',
      pl: 'Uruchamiaj AigisCode w dowolnym systemie CI, aby wychwytywac problemy architektoniczne, zanim trafa do produkcji. Obsluguje kody wyjscia, artefakty JSON i flage --skip-ai dla analizy deterministycznej.',
      bn: 'যেকোনো CI সিস্টেমে AigisCode চালান আর্কিটেকচারাল সমস্যা প্রোডাকশনে পৌঁছানোর আগে ধরতে। এক্সিট কোড, JSON আর্টিফ্যাক্ট এবং ডিটার্মিনিস্টিক বিশ্লেষণের জন্য --skip-ai ফ্ল্যাগ সাপোর্ট করে।',
    },
    metaDescription: {
      en: 'Integrate AigisCode into your CI/CD pipeline. Catch circular dependencies, dead code, and hardwired values automatically on every push. Works with any CI system.',
      cs: 'Integrujte AigisCode do vašeho CI/CD pipeline. Automaticky zachytávejte cyklické závislosti, mrtvý kód a natvrdo zapsané hodnoty při každém push. Funguje s jakýmkoli CI systémem.',
      fr: 'Intégrez AigisCode dans votre pipeline CI/CD. Détectez automatiquement les dépendances circulaires, le code mort et les valeurs codées en dur à chaque push. Compatible avec tout système CI.',
      es: 'Integre AigisCode en su pipeline CI/CD. Detecte automáticamente dependencias circulares, código muerto y valores codificados en cada push. Compatible con cualquier sistema CI.',
      zh: '将 AigisCode 集成到您的 CI/CD 流水线中。每次推送时自动捕获循环依赖、死代码和硬编码值。兼容任何 CI 系统。',
      hi: 'AigisCode को अपने CI/CD पाइपलाइन में इंटीग्रेट करें। हर पुश पर स्वचालित रूप से सर्कुलर डिपेंडेंसी, डेड कोड और हार्डवायर्ड वैल्यू पकड़ें। किसी भी CI सिस्टम के साथ काम करता है।',
      pt: 'Integre o AigisCode ao seu pipeline CI/CD. Capture automaticamente dependências circulares, código morto e valores codificados a cada push. Funciona com qualquer sistema CI.',
      ar: 'ادمج AigisCode في خط أنابيب CI/CD. اكتشف التبعيات الدائرية والشيفرة الميتة والقيم الثابتة تلقائياً مع كل دفع. يعمل مع أي نظام CI.',
      pl: 'Zintegruj AigisCode z Twoim pipeline CI/CD. Automatycznie wychwytuj cykliczne zaleznosci, martwy kod i zakodowane wartosci przy kazdym pushu. Dziala z dowolnym systemem CI.',
      bn: 'AigisCode আপনার CI/CD pipeline-এ ইন্টিগ্রেট করুন। প্রতিটি push-এ স্বয়ংক্রিয়ভাবে সার্কুলার ডিপেন্ডেন্সি, ডেড কোড এবং হার্ডওয়্যার্ড ভ্যালু ধরুন। যেকোনো CI সিস্টেমের সাথে কাজ করে।',
    },
    content: {
      en: `<h2 id="why-ci-cd">Why Run AigisCode in CI/CD?</h2>
<p>Static analysis is most effective when it runs automatically on every push. Developers forget to run tools locally, and manual reviews miss structural issues that span dozens of files. By integrating AigisCode into your CI/CD pipeline, you create a permanent quality gate that catches circular dependencies, dead code, and hardwired values before they accumulate.</p>
<p>Unlike single-file linters that only check syntax and style, AigisCode analyzes your <strong>entire dependency graph</strong>. This means it catches problems that are invisible at the file level — architectural cycles, orphan modules, god classes, and bottleneck files that create fragile coupling.</p>

<h2 id="generic-ci-setup">Generic CI Setup</h2>
<p>AigisCode works with any CI system that can run the native Rust CLI. The integration pattern is the same everywhere:</p>
<ol>
<li><strong>Install</strong> — <code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code> in your CI environment.</li>
<li><strong>Run</strong> — Execute <code>aigiscode analyze .</code> from the repository root.</li>
<li><strong>Check exit code</strong> — AigisCode returns exit code 0 for a clean analysis and non-zero when findings exceed your configured thresholds.</li>
<li><strong>Archive artifacts</strong> — The JSON report at <code>.aigiscode/deterministic-analysis.json</code> can be uploaded as a build artifact for later inspection.</li>
</ol>
<p>Here is a minimal pipeline step that works in virtually any CI system:</p>
<pre><code># Install and run AigisCode
cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
aigiscode analyze .

# The exit code signals pass/fail to your CI system
# Upload .aigiscode/deterministic-analysis.json as a build artifact</code></pre>

<h2 id="skip-ai-flag">The --skip-ai Flag</h2>
<p>By default, AigisCode uses AI backends (OpenAI, Anthropic) for its final-stage review classification. In CI environments, you often want <strong>deterministic, reproducible results</strong> without API calls. The <code>--skip-ai</code> flag disables all AI backends and runs only the deterministic detectors:</p>
<ul>
<li>Circular dependency detection (Tarjan SCC analysis)</li>
<li>Dead code detection (unused imports, unreferenced methods, orphaned properties)</li>
<li>Hardwired value detection (magic strings, repeated literals, hardcoded network addresses)</li>
<li>God class detection, bottleneck analysis, layer violation checking</li>
</ul>
<p>This makes your CI pipeline faster, cheaper, and fully reproducible. You can still run the full AI-assisted analysis locally or in a separate scheduled pipeline.</p>

<h2 id="exit-codes">Exit Codes and Thresholds</h2>
<p>AigisCode uses standard exit codes to integrate with CI systems:</p>
<ul>
<li><strong>Exit 0</strong> — Analysis completed, no findings above threshold.</li>
<li><strong>Exit 1</strong> — Analysis completed, findings exceed configured thresholds.</li>
<li><strong>Exit 2</strong> — Analysis failed (parse error, missing dependencies, configuration error).</li>
</ul>
<p>You can configure thresholds in your <code>.aigiscode/policy.json</code> to control what triggers a failure. For example, you might allow up to 5 dead imports but fail on any strong circular dependency.</p>

<h2 id="artifact-handling">Artifact Handling</h2>
<p>The primary output is <code>.aigiscode/deterministic-analysis.json</code>, a machine-readable JSON file containing all findings. Timestamped archives are also written to <code>.aigiscode/reports/</code> for historical comparison. Upload these as CI artifacts to enable:</p>
<ul>
<li>Trend tracking across builds</li>
<li>PR comment bots that summarize new findings</li>
<li>Dashboard integrations that visualize architectural health over time</li>
<li>Diff analysis between the current and previous report</li>
</ul>`,
      cs: `<h2 id="why-ci-cd">Proč spouštět AigisCode v CI/CD?</h2>
<p>Statická analýza je nejefektivnější, když běží automaticky při každém push. Integrací AigisCode do CI/CD pipeline vytvoříte trvalou kvalitní bránu, která zachytí cyklické závislosti, mrtvý kód a hardwired hodnoty dříve, než se nahromadí.</p>

<h2 id="generic-ci-setup">Obecné CI nastavení</h2>
<p>AigisCode funguje s jakýmkoli CI systémem, který umí spustit Rust CLI. Vzor integrace: nainstalujte (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>), spusťte (<code>aigiscode analyze .</code>), zkontrolujte návratový kód a archivujte artefakty (<code>.aigiscode/deterministic-analysis.json</code>).</p>

<h2 id="skip-ai-flag">Příznak --skip-ai</h2>
<p>V CI prostředí často chcete deterministické, reprodukovatelné výsledky bez API volání. Příznak <code>--skip-ai</code> deaktivuje všechny AI backendy a spustí pouze deterministické detektory.</p>

<h2 id="exit-codes">Návratové kódy</h2>
<ul>
<li><strong>Exit 0</strong> — Analýza dokončena, žádné nálezy nad prahem.</li>
<li><strong>Exit 1</strong> — Analýza dokončena, nálezy přesahují práh.</li>
<li><strong>Exit 2</strong> — Analýza selhala.</li>
</ul>`,
      fr: `<h2 id="why-ci-cd">Pourquoi executer AigisCode en CI/CD ?</h2>
<p>L'analyse statique est plus efficace quand elle s'execute automatiquement a chaque push. En integrant AigisCode dans votre pipeline CI/CD, vous creez une porte qualite permanente qui detecte les dependances circulaires, le code mort et les valeurs codees en dur avant qu'ils ne s'accumulent.</p>

<h2 id="generic-ci-setup">Configuration CI generique</h2>
<p>AigisCode fonctionne avec tout systeme CI capable d'executer le CLI Rust. Installez (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>), executez (<code>aigiscode analyze .</code>), verifiez le code de sortie et archivez les artefacts.</p>

<h2 id="skip-ai-flag">Le drapeau --skip-ai</h2>
<p>En CI, vous voulez souvent des resultats deterministes et reproductibles sans appels API. Le drapeau <code>--skip-ai</code> desactive tous les backends IA.</p>

<h2 id="exit-codes">Codes de sortie</h2>
<ul>
<li><strong>Exit 0</strong> — Analyse terminee, aucun resultat au-dessus du seuil.</li>
<li><strong>Exit 1</strong> — Analyse terminee, resultats depassant les seuils.</li>
<li><strong>Exit 2</strong> — Analyse echouee.</li>
</ul>`,
      es: `<h2 id="why-ci-cd">Por que ejecutar AigisCode en CI/CD?</h2>
<p>El analisis estatico es mas efectivo cuando se ejecuta automaticamente en cada push. Al integrar AigisCode en su pipeline CI/CD, crea una puerta de calidad permanente que detecta dependencias circulares, codigo muerto y valores hardcoded antes de que se acumulen.</p>

<h2 id="generic-ci-setup">Configuracion CI generica</h2>
<p>AigisCode funciona con cualquier sistema CI que pueda ejecutar el CLI Rust. Instale (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>), ejecute (<code>aigiscode analyze .</code>), verifique el codigo de salida y archive artefactos.</p>

<h2 id="skip-ai-flag">La bandera --skip-ai</h2>
<p>En CI, a menudo quiere resultados deterministicos y reproducibles sin llamadas API. La bandera <code>--skip-ai</code> desactiva todos los backends de IA.</p>

<h2 id="exit-codes">Codigos de salida</h2>
<ul>
<li><strong>Exit 0</strong> — Analisis completado, sin hallazgos sobre el umbral.</li>
<li><strong>Exit 1</strong> — Analisis completado, hallazgos exceden umbrales.</li>
<li><strong>Exit 2</strong> — Analisis fallido.</li>
</ul>`,
      zh: `<h2 id="why-ci-cd">为什么在CI/CD中运行AigisCode？</h2>
<p>静态分析在每次推送时自动运行时最为有效。将AigisCode集成到CI/CD流水线中，可创建永久的质量门控，在循环依赖、死代码和硬编码值积累之前捕获它们。</p>

<h2 id="generic-ci-setup">通用CI设置</h2>
<p>AigisCode可与任何能运行Rust CLI的CI系统配合。安装（<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>）、运行（<code>aigiscode analyze .</code>）、检查退出码并归档产物。</p>

<h2 id="skip-ai-flag">--skip-ai标志</h2>
<p>在CI环境中，通常需要无API调用的确定性、可重现结果。<code>--skip-ai</code>标志禁用所有AI后端。</p>

<h2 id="exit-codes">退出码</h2>
<ul>
<li><strong>Exit 0</strong> — 分析完成，无超出阈值的发现。</li>
<li><strong>Exit 1</strong> — 分析完成，发现超出配置的阈值。</li>
<li><strong>Exit 2</strong> — 分析失败。</li>
</ul>`,
      hi: `<h2 id="why-ci-cd">CI/CD में AigisCode क्यों चलाएं?</h2>
<p>स्टैटिक एनालिसिस सबसे प्रभावी तब होता है जब यह हर push पर स्वचालित रूप से चलता है। AigisCode को CI/CD पाइपलाइन में इंटीग्रेट करके, आप एक स्थायी क्वालिटी गेट बनाते हैं।</p>

<h2 id="generic-ci-setup">जेनेरिक CI सेटअप</h2>
<p>AigisCode किसी भी CI सिस्टम के साथ काम करता है जो Rust CLI चला सके। इंस्टॉल करें (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>), चलाएं (<code>aigiscode analyze .</code>), एग्ज़िट कोड चेक करें और आर्टिफैक्ट आर्काइव करें।</p>

<h2 id="skip-ai-flag">--skip-ai फ्लैग</h2>
<p>CI में, अक्सर API कॉल के बिना डिटर्मिनिस्टिक, रिप्रोड्यूसिबल रिजल्ट चाहिए। <code>--skip-ai</code> फ्लैग सभी AI बैकएंड को डिसेबल करता है।</p>

<h2 id="exit-codes">एग्ज़िट कोड</h2>
<ul>
<li><strong>Exit 0</strong> — एनालिसिस पूर्ण, थ्रेशोल्ड से ऊपर कोई फाइंडिंग नहीं।</li>
<li><strong>Exit 1</strong> — एनालिसिस पूर्ण, फाइंडिंग थ्रेशोल्ड से अधिक।</li>
<li><strong>Exit 2</strong> — एनालिसिस विफल।</li>
</ul>`,
      pt: `<h2 id="why-ci-cd">Por que executar AigisCode em CI/CD?</h2>
<p>A analise estatica e mais eficaz quando executada automaticamente em cada push. Ao integrar AigisCode no pipeline CI/CD, voce cria um portao de qualidade permanente que detecta dependencias circulares, codigo morto e valores hardcoded antes que se acumulem.</p>

<h2 id="generic-ci-setup">Configuracao CI generica</h2>
<p>AigisCode funciona com qualquer sistema CI que possa executar o CLI Rust. Instale (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>), execute (<code>aigiscode analyze .</code>), verifique o codigo de saida e arquive artefatos.</p>

<h2 id="skip-ai-flag">A flag --skip-ai</h2>
<p>Em CI, frequentemente voce quer resultados deterministicos e reproduziveis sem chamadas de API. A flag <code>--skip-ai</code> desabilita todos os backends de IA.</p>

<h2 id="exit-codes">Codigos de saida</h2>
<ul>
<li><strong>Exit 0</strong> — Analise concluida, sem descobertas acima do limite.</li>
<li><strong>Exit 1</strong> — Analise concluida, descobertas excedem limites.</li>
<li><strong>Exit 2</strong> — Analise falhou.</li>
</ul>`,
      ar: `<h2 id="why-ci-cd">لماذا تشغّل AigisCode في CI/CD؟</h2>
<p>التحليل الثابت أكثر فعالية عند تشغيله تلقائياً مع كل دفع. بدمج AigisCode في خط أنابيب CI/CD تُنشئ بوابة جودة دائمة تكتشف التبعيات الدائرية والشيفرة الميتة والقيم الثابتة قبل تراكمها.</p>

<h2 id="generic-ci-setup">إعداد CI عام</h2>
<p>يعمل AigisCode مع أي نظام CI يمكنه تشغيل واجهة Rust. ثبّت (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>) وشغّل (<code>aigiscode analyze .</code>) وتحقق من كود الخروج وأرشف المخرجات.</p>

<h2 id="skip-ai-flag">علامة --skip-ai</h2>
<p>في CI غالباً تريد نتائج حتمية وقابلة للتكرار بدون استدعاءات API. علامة <code>--skip-ai</code> تعطّل جميع واجهات الذكاء الاصطناعي.</p>

<h2 id="exit-codes">أكواد الخروج</h2>
<ul>
<li><strong>الخروج 0</strong> — اكتمل التحليل بدون نتائج فوق العتبة.</li>
<li><strong>الخروج 1</strong> — اكتمل التحليل والنتائج تتجاوز العتبات.</li>
<li><strong>الخروج 2</strong> — فشل التحليل.</li>
</ul>`,
      pl: `<h2 id="why-ci-cd">Dlaczego uruchamiac AigisCode w CI/CD?</h2>
<p>Analiza statyczna jest najbardziej efektywna automatycznie przy kazdym pushu.</p>
<h2 id="generic-ci-setup">Ogolna konfiguracja CI</h2>
<p>Zainstaluj (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>), uruchom (<code>aigiscode analyze .</code>), sprawdz kod wyjscia i zarchiwizuj artefakty.</p>
<h2 id="exit-codes">Kody wyjscia</h2>
<ul>
<li><strong>Exit 0</strong> — Analiza zakonczona, brak znalezisk powyzej progu.</li>
<li><strong>Exit 1</strong> — Analiza zakonczona, znaleziska przekraczaja progi.</li>
<li><strong>Exit 2</strong> — Analiza nie powiodla sie.</li>
</ul>`,
    },
    features: [
      'Works with any CI system',
      'Deterministic --skip-ai mode',
      'Standard exit codes',
      'JSON artifact output',
      'Configurable thresholds',
      'Timestamped report archives',
    ],
    codeExample: `# Generic CI pipeline step
cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode

# Run deterministic analysis (no AI API calls)
aigiscode analyze .

# Upload the report as a build artifact
# Path: .aigiscode/deterministic-analysis.json`,
    relatedSlugs: ['github-actions', 'gitlab-ci', 'pre-commit-hooks'],
  },

  /* ---- 2. GitHub Actions ---- */
  {
    slug: 'github-actions',
    icon: 'GithubLogo',
    category: 'CI/CD',
    status: 'available',
    title: {
      en: 'GitHub Actions',
      cs: 'GitHub Actions',
      fr: 'GitHub Actions',
      es: 'GitHub Actions',
      zh: 'GitHub Actions',
      hi: 'GitHub Actions',
      pt: 'GitHub Actions',
      ar: 'GitHub Actions',
      pl: 'GitHub Actions',
      bn: 'GitHub Actions',
    },
    shortDescription: {
      en: 'Pre-built workflow for GitHub Actions that runs AigisCode on every pull request, posts a summary comment, and uploads the JSON report as an artifact.',
      cs: 'Předpřipravený workflow pro GitHub Actions, který spouští AigisCode u každého pull requestu, zveřejňuje souhrnný komentář a nahrává JSON report jako artefakt.',
      fr: 'Workflow prêt à l\'emploi pour GitHub Actions qui exécute AigisCode sur chaque pull request, publie un commentaire récapitulatif et télécharge le rapport JSON en tant qu\'artefact.',
      es: 'Workflow prediseñado para GitHub Actions que ejecuta AigisCode en cada pull request, publica un comentario resumen y sube el informe JSON como artefacto.',
      zh: '预构建的 GitHub Actions 工作流，在每个 Pull Request 上运行 AigisCode，发布摘要评论并将 JSON 报告上传为产物。',
      hi: 'GitHub Actions के लिए पूर्व-निर्मित वर्कफ़्लो जो हर पुल रिक्वेस्ट पर AigisCode चलाता है, सारांश कमेंट पोस्ट करता है और JSON रिपोर्ट को आर्टिफैक्ट के रूप में अपलोड करता है।',
      pt: 'Workflow pronto para GitHub Actions que executa o AigisCode em cada pull request, publica um comentário resumo e faz upload do relatório JSON como artefato.',
      ar: 'سير عمل جاهز لـ GitHub Actions يشغّل AigisCode على كل طلب سحب وينشر تعليقاً ملخصاً ويرفع تقرير JSON كمخرج.',
      pl: 'Gotowy workflow dla GitHub Actions, ktory uruchamia AigisCode przy kazdym pull requeście, publikuje komentarz podsumowujacy ze znaleziskami i blokuje merge, gdy progi sa przekroczone.',
      bn: 'প্রি-বিল্ট GitHub Actions ওয়ার্কফ্লো যা প্রতিটি pull request-এ AigisCode চালায়, সারাংশ কমেন্ট পোস্ট করে এবং JSON রিপোর্ট আর্টিফ্যাক্ট হিসেবে আপলোড করে।',
    },
    metaDescription: {
      en: 'Run AigisCode in GitHub Actions. Pre-built workflow YAML for pull request analysis, comment integration, and artifact uploads. Catch architecture issues on every PR.',
      cs: 'Spouštějte AigisCode v GitHub Actions. Předpřipravený workflow YAML pro analýzu pull requestů, integraci komentářů a nahrávání artefaktů. Zachyťte architektonické problémy u každého PR.',
      fr: 'Exécutez AigisCode dans GitHub Actions. Workflow YAML prêt à l\'emploi pour l\'analyse des pull requests, l\'intégration des commentaires et le téléchargement des artefacts. Détectez les problèmes d\'architecture à chaque PR.',
      es: 'Ejecute AigisCode en GitHub Actions. Workflow YAML prediseñado para análisis de pull requests, integración de comentarios y subida de artefactos. Detecte problemas de arquitectura en cada PR.',
      zh: '在 GitHub Actions 中运行 AigisCode。预构建的工作流 YAML，用于 Pull Request 分析、评论集成和产物上传。在每个 PR 中捕获架构问题。',
      hi: 'GitHub Actions में AigisCode चलाएं। पुल रिक्वेस्ट विश्लेषण, कमेंट इंटीग्रेशन और आर्टिफैक्ट अपलोड के लिए पूर्व-निर्मित वर्कफ़्लो YAML। हर PR पर आर्किटेक्चर समस्याएं पकड़ें।',
      pt: 'Execute o AigisCode no GitHub Actions. Workflow YAML pronto para análise de pull requests, integração de comentários e upload de artefatos. Capture problemas de arquitetura em cada PR.',
      ar: 'شغّل AigisCode في GitHub Actions. ملف YAML جاهز لسير عمل تحليل طلبات السحب وتكامل التعليقات ورفع المخرجات. اكتشف المشكلات المعمارية في كل طلب سحب.',
      pl: 'Uruchom AigisCode w GitHub Actions. Gotowy plik YAML workflow do analizy pull requestow, integracji komentarzy i blokowania merge opartego na progach.',
      bn: 'GitHub Actions-এ AigisCode চালান। pull request বিশ্লেষণ, কমেন্ট ইন্টিগ্রেশন এবং আর্টিফ্যাক্ট আপলোডের জন্য প্রি-বিল্ট ওয়ার্কফ্লো YAML। প্রতিটি PR-এ আর্কিটেকচার সমস্যা ধরুন।',
    },
    content: {
      en: `<h2 id="github-actions-overview">AigisCode in GitHub Actions</h2>
<p>GitHub Actions is the most popular CI/CD platform for open-source projects and a natural fit for AigisCode. With a single workflow file, you can run architectural analysis on every pull request and surface findings directly in the PR conversation.</p>
<p>The workflow installs AigisCode, runs the analysis in deterministic mode, uploads the JSON report as a build artifact, and optionally posts a comment summarizing the findings. This gives reviewers immediate visibility into structural issues without leaving the GitHub UI.</p>

<h2 id="workflow-yaml">Workflow Configuration</h2>
<p>Create <code>.github/workflows/aigiscode.yml</code> in your repository:</p>
<pre><code>name: AigisCode Analysis

on:
  pull_request:
    branches: [main, develop]
  push:
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

      - name: Run analysis
        run: aigiscode analyze .
        continue-on-error: true

      - name: Upload report
        uses: actions/upload-artifact@v4
        with:
          name: aigiscode-report
          path: .aigiscode/deterministic-analysis.json
          retention-days: 30</code></pre>

<h2 id="pr-comment-integration">Pull Request Comment Integration</h2>
<p>You can extend the workflow to post a summary comment on the pull request. This uses the JSON report to extract key metrics and findings, then posts them using the GitHub API:</p>
<pre><code>      - name: Post PR comment
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = JSON.parse(
              fs.readFileSync('.aigiscode/deterministic-analysis.json', 'utf8')
            );
            const summary = report.summary;
            const body = [
              '## AigisCode Analysis',
              '',
              \`| Metric | Count |\`,
              \`|--------|-------|\`,
              \`| Circular dependencies | \${summary.circular_dependencies || 0} |\`,
              \`| Dead code findings | \${summary.dead_code || 0} |\`,
              \`| Hardwired values | \${summary.hardwired_values || 0} |\`,
              \`| God classes | \${summary.god_classes || 0} |\`,
              '',
              '*Generated by [AigisCode](https://aigiscode.com)*',
            ].join('\\n');

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body,
            });</code></pre>

<h2 id="branch-protection">Branch Protection Rules</h2>
<p>Once the workflow is in place, you can require it to pass before merging. Go to your repository settings, navigate to <strong>Branch protection rules</strong>, and add the <code>analyze</code> job as a required status check. This ensures no PR can be merged if it introduces new architectural issues above your configured thresholds.</p>

<h2 id="caching">Caching for Faster Runs</h2>
<p>AigisCode parses your entire codebase on each run. For large repositories, you can speed up the pipeline by caching the Python installation and AigisCode package:</p>
<pre><code>      - name: Cache pip packages
        uses: actions/cache@v4
        with:
          path: ~/.cache/pip
          key: \${{ runner.os }}-pip-aigiscode
          restore-keys: \${{ runner.os }}-pip-</code></pre>
<p>This typically reduces installation time from 15-30 seconds to under 2 seconds on cache hits.</p>`,
      cs: `<h2 id="github-actions-overview">AigisCode v GitHub Actions</h2>
<p>GitHub Actions je nejpopulárnější CI/CD platforma pro open-source projekty. Jediným workflow souborem můžete spustit architektonickou analýzu u každého pull requestu a zobrazit nálezy přímo v PR konverzaci.</p>

<h2 id="workflow-yaml">Konfigurace workflow</h2>
<p>Vytvořte <code>.github/workflows/aigiscode.yml</code> ve svém repozitáři. Workflow nainstaluje AigisCode, spustí analýzu v deterministickém režimu, nahraje JSON report jako artefakt a volitelně přidá komentář shrnující nálezy.</p>

<h2 id="pr-comment-integration">Integrace komentářů k Pull Requestům</h2>
<p>Rozšiřte workflow o zveřejnění souhrnného komentáře pomocí <code>actions/github-script</code>. Komentář obsahuje tabulku s klíčovými metrikami: cyklické závislosti, mrtvý kód, hardwired hodnoty a god classes.</p>

<h2 id="branch-protection">Ochrana větví</h2>
<p>Přidejte úlohu <code>analyze</code> jako povinnou kontrolu stavu v nastavení ochrany větví, aby žádný PR nemohl být sloučen s novými architektonickými problémy.</p>

<h2 id="caching">Cachování pro rychlejší běhy</h2>
<p>Použijte <code>actions/cache</code> pro cachování pip balíčků. To obvykle zkrátí dobu instalace z 15-30 sekund na méně než 2 sekundy.</p>`,
      fr: `<h2 id="github-actions-overview">AigisCode dans GitHub Actions</h2>
<p>GitHub Actions est la plateforme CI/CD la plus populaire pour les projets open-source. Avec un seul fichier workflow, vous pouvez executer l'analyse architecturale sur chaque pull request et afficher les resultats directement dans la conversation PR.</p>

<h2 id="workflow-yaml">Configuration du workflow</h2>
<p>Creez <code>.github/workflows/aigiscode.yml</code> dans votre depot. Le workflow installe AigisCode, execute l'analyse en mode deterministe, telecharge le rapport JSON comme artefact et publie optionnellement un commentaire resumant les resultats.</p>

<h2 id="pr-comment-integration">Integration des commentaires PR</h2>
<p>Etendez le workflow pour publier un commentaire resume via <code>actions/github-script</code> avec un tableau des metriques cles.</p>

<h2 id="branch-protection">Regles de protection de branche</h2>
<p>Ajoutez le job <code>analyze</code> comme verification de statut requise dans les parametres de protection de branche.</p>

<h2 id="caching">Cache pour des executions plus rapides</h2>
<p>Utilisez <code>actions/cache</code> pour mettre en cache les paquets pip. Cela reduit le temps d'installation de 15-30 secondes a moins de 2 secondes.</p>`,
      es: `<h2 id="github-actions-overview">AigisCode en GitHub Actions</h2>
<p>GitHub Actions es la plataforma CI/CD mas popular para proyectos de codigo abierto. Con un solo archivo workflow, puede ejecutar analisis arquitectonico en cada pull request y mostrar hallazgos directamente en la conversacion del PR.</p>

<h2 id="workflow-yaml">Configuracion del workflow</h2>
<p>Cree <code>.github/workflows/aigiscode.yml</code> en su repositorio. El workflow instala AigisCode, ejecuta el analisis en modo deterministico, sube el informe JSON como artefacto y opcionalmente publica un comentario resumiendo los hallazgos.</p>

<h2 id="pr-comment-integration">Integracion de comentarios PR</h2>
<p>Extienda el workflow para publicar un comentario resumen via <code>actions/github-script</code> con una tabla de metricas clave.</p>

<h2 id="branch-protection">Reglas de proteccion de ramas</h2>
<p>Agregue el job <code>analyze</code> como verificacion de estado requerida en la configuracion de proteccion de ramas.</p>

<h2 id="caching">Cache para ejecuciones mas rapidas</h2>
<p>Use <code>actions/cache</code> para cachear paquetes pip. Esto reduce el tiempo de instalacion de 15-30 segundos a menos de 2 segundos.</p>`,
      zh: `<h2 id="github-actions-overview">GitHub Actions 中的 AigisCode</h2>
<p>GitHub Actions 是最受欢迎的开源项目 CI/CD 平台。通过一个工作流文件，您可以在每个 Pull Request 上运行架构分析，并直接在 PR 对话中展示发现结果。</p>

<h2 id="workflow-yaml">工作流配置</h2>
<p>在仓库中创建 <code>.github/workflows/aigiscode.yml</code>。工作流安装 AigisCode，以确定性模式运行分析，将 JSON 报告上传为产物，并可选地发布发现结果摘要评论。</p>

<h2 id="pr-comment-integration">Pull Request 评论集成</h2>
<p>扩展工作流，通过 <code>actions/github-script</code> 发布包含关键指标表格的摘要评论。</p>

<h2 id="branch-protection">分支保护规则</h2>
<p>在分支保护设置中将 <code>analyze</code> 作业添加为必需的状态检查。</p>

<h2 id="caching">缓存加速</h2>
<p>使用 <code>actions/cache</code> 缓存 pip 包。这通常将安装时间从 15-30 秒缩短到不到 2 秒。</p>`,
      hi: `<h2 id="github-actions-overview">GitHub Actions में AigisCode</h2>
<p>GitHub Actions ओपन-सोर्स प्रोजेक्ट्स के लिए सबसे लोकप्रिय CI/CD प्लेटफ़ॉर्म है। एक वर्कफ़्लो फ़ाइल से, आप हर पुल रिक्वेस्ट पर आर्किटेक्चरल एनालिसिस चला सकते हैं और PR कन्वर्सेशन में सीधे फाइंडिंग दिखा सकते हैं।</p>

<h2 id="workflow-yaml">वर्कफ़्लो कॉन्फ़िगरेशन</h2>
<p>अपने रिपॉज़िटरी में <code>.github/workflows/aigiscode.yml</code> बनाएं। वर्कफ़्लो AigisCode इंस्टॉल करता है, डिटर्मिनिस्टिक मोड में एनालिसिस चलाता है, JSON रिपोर्ट को आर्टिफैक्ट के रूप में अपलोड करता है।</p>

<h2 id="pr-comment-integration">PR कमेंट इंटीग्रेशन</h2>
<p><code>actions/github-script</code> के माध्यम से मुख्य मेट्रिक्स तालिका के साथ सारांश कमेंट पोस्ट करने के लिए वर्कफ़्लो का विस्तार करें।</p>

<h2 id="branch-protection">ब्रांच प्रोटेक्शन नियम</h2>
<p>ब्रांच प्रोटेक्शन सेटिंग्स में <code>analyze</code> जॉब को आवश्यक स्टेटस चेक के रूप में जोड़ें।</p>

<h2 id="caching">तेज़ रन के लिए कैशिंग</h2>
<p>pip पैकेज कैश करने के लिए <code>actions/cache</code> का उपयोग करें। यह इंस्टॉलेशन समय को 15-30 सेकंड से घटाकर 2 सेकंड से कम कर देता है।</p>`,
      pt: `<h2 id="github-actions-overview">AigisCode no GitHub Actions</h2>
<p>GitHub Actions e a plataforma CI/CD mais popular para projetos open-source. Com um unico arquivo workflow, voce pode executar analise arquitetural em cada pull request e exibir descobertas diretamente na conversa do PR.</p>

<h2 id="workflow-yaml">Configuracao do workflow</h2>
<p>Crie <code>.github/workflows/aigiscode.yml</code> no seu repositorio. O workflow instala o AigisCode, executa a analise em modo deterministico, faz upload do relatorio JSON como artefato e opcionalmente publica um comentario resumindo as descobertas.</p>

<h2 id="pr-comment-integration">Integracao de comentarios PR</h2>
<p>Estenda o workflow para publicar um comentario resumo via <code>actions/github-script</code> com uma tabela de metricas-chave.</p>

<h2 id="branch-protection">Regras de protecao de branch</h2>
<p>Adicione o job <code>analyze</code> como verificacao de status obrigatoria nas configuracoes de protecao de branch.</p>

<h2 id="caching">Cache para execucoes mais rapidas</h2>
<p>Use <code>actions/cache</code> para cachear pacotes pip. Isso reduz o tempo de instalacao de 15-30 segundos para menos de 2 segundos.</p>`,
      ar: `<h2 id="github-actions-overview">AigisCode في GitHub Actions</h2>
<p>GitHub Actions هي أشهر منصة CI/CD للمشاريع مفتوحة المصدر. بملف سير عمل واحد يمكنك تشغيل التحليل المعماري في كل طلب سحب وعرض النتائج مباشرة في محادثة الطلب.</p>

<h2 id="workflow-yaml">تكوين سير العمل</h2>
<p>أنشئ <code>.github/workflows/aigiscode.yml</code> في مستودعك. يثبّت سير العمل AigisCode ويشغّل التحليل بالوضع الحتمي ويرفع تقرير JSON كمخرج ويمكنه اختيارياً نشر تعليق يلخّص النتائج.</p>

<h2 id="pr-comment-integration">تكامل تعليقات طلبات السحب</h2>
<p>وسّع سير العمل لنشر تعليق ملخص عبر <code>actions/github-script</code> مع جدول للمقاييس الرئيسية.</p>

<h2 id="branch-protection">قواعد حماية الفرع</h2>
<p>أضف مهمة <code>analyze</code> كفحص حالة إلزامي في إعدادات حماية الفرع.</p>

<h2 id="caching">التخزين المؤقت لتشغيل أسرع</h2>
<p>استخدم <code>actions/cache</code> لتخزين حزم pip مؤقتاً. يقلل هذا وقت التثبيت من ١٥-٣٠ ثانية إلى أقل من ٢ ثانية.</p>`,
      pl: `<h2 id="github-actions">GitHub Actions</h2>
<p>Gotowy workflow YAML dla GitHub Actions. Instaluje AigisCode, uruchamia analize z <code>--skip-ai</code> i publikuje komentarz podsumowujacy.</p>`,
    },
    features: [
      'Pre-built workflow YAML',
      'Pull request comments',
      'Artifact upload',
      'Branch protection support',
      'Pip caching for speed',
      'Deterministic --skip-ai mode',
    ],
    codeExample: `# .github/workflows/aigiscode.yml
name: AigisCode Analysis
on:
  pull_request:
    branches: [main]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - run: cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
      - run: aigiscode analyze .
      - uses: actions/upload-artifact@v4
        with:
          name: aigiscode-report
          path: .aigiscode/deterministic-analysis.json`,
    relatedSlugs: ['ci-cd-pipelines', 'gitlab-ci', 'pre-commit-hooks'],
  },

  /* ---- 3. GitLab CI ---- */
  {
    slug: 'gitlab-ci',
    icon: 'Circuitry',
    category: 'CI/CD',
    status: 'available',
    title: {
      en: 'GitLab CI',
      cs: 'GitLab CI',
      fr: 'GitLab CI',
      es: 'GitLab CI',
      zh: 'GitLab CI',
      hi: 'GitLab CI',
      pt: 'GitLab CI',
      ar: 'GitLab CI',
      pl: 'GitLab CI',
      bn: 'GitLab CI',
    },
    shortDescription: {
      en: 'GitLab CI template for running AigisCode in your pipeline. Includes job configuration, artifact handling, and merge request integration.',
      cs: 'Šablona pro GitLab CI ke spuštění AigisCode ve vašem pipeline. Zahrnuje konfiguraci úlohy, správu artefaktů a integraci s merge requesty.',
      fr: 'Template GitLab CI pour exécuter AigisCode dans votre pipeline. Inclut la configuration des jobs, la gestion des artefacts et l\'intégration avec les merge requests.',
      es: 'Plantilla de GitLab CI para ejecutar AigisCode en su pipeline. Incluye configuración de jobs, manejo de artefactos e integración con merge requests.',
      zh: 'GitLab CI 模板，用于在您的流水线中运行 AigisCode。包括作业配置、产物处理和合并请求集成。',
      hi: 'आपके पाइपलाइन में AigisCode चलाने के लिए GitLab CI टेम्प्लेट। जॉब कॉन्फ़िगरेशन, आर्टिफैक्ट हैंडलिंग और मर्ज रिक्वेस्ट इंटीग्रेशन शामिल है।',
      pt: 'Template do GitLab CI para executar o AigisCode no seu pipeline. Inclui configuração de jobs, gerenciamento de artefatos e integração com merge requests.',
      ar: 'قالب GitLab CI لتشغيل AigisCode في خط الأنابيب. يتضمن تكوين المهام وإدارة المخرجات والتكامل مع طلبات الدمج.',
      pl: 'Szablon GitLab CI do uruchamiania AigisCode w Twoim pipeline. Obejmuje konfiguracje zadan, obsluge artefaktow, komentarze do merge requestow i flage --skip-ai dla powtarzalnych wynikow.',
      bn: 'আপনার pipeline-এ AigisCode চালানোর জন্য GitLab CI টেমপ্লেট। জব কনফিগারেশন, আর্টিফ্যাক্ট হ্যান্ডলিং এবং merge request ইন্টিগ্রেশন অন্তর্ভুক্ত।',
    },
    metaDescription: {
      en: 'Integrate AigisCode with GitLab CI. Ready-to-use .gitlab-ci.yml template with artifact handling, merge request analysis, and configurable thresholds.',
      cs: 'Integrujte AigisCode s GitLab CI. Připravená šablona .gitlab-ci.yml se správou artefaktů, analýzou merge requestů a konfigurovatelnými prahovými hodnotami.',
      fr: 'Intégrez AigisCode avec GitLab CI. Template .gitlab-ci.yml prêt à l\'emploi avec gestion des artefacts, analyse des merge requests et seuils configurables.',
      es: 'Integre AigisCode con GitLab CI. Plantilla .gitlab-ci.yml lista para usar con manejo de artefactos, análisis de merge requests y umbrales configurables.',
      zh: '将 AigisCode 与 GitLab CI 集成。即用型 .gitlab-ci.yml 模板，包含产物处理、合并请求分析和可配置阈值。',
      hi: 'AigisCode को GitLab CI के साथ इंटीग्रेट करें। आर्टिफैक्ट हैंडलिंग, मर्ज रिक्वेस्ट विश्लेषण और कॉन्फ़िगर करने योग्य थ्रेशोल्ड के साथ तैयार .gitlab-ci.yml टेम्प्लेट।',
      pt: 'Integre o AigisCode com o GitLab CI. Template .gitlab-ci.yml pronto para uso com gerenciamento de artefatos, análise de merge requests e limites configuráveis.',
      ar: 'ادمج AigisCode مع GitLab CI. قالب .gitlab-ci.yml جاهز للاستخدام مع إدارة المخرجات وتحليل طلبات الدمج وعتبات قابلة للتكوين.',
      pl: 'Zintegruj AigisCode z GitLab CI. Gotowy szablon .gitlab-ci.yml z obsluga artefaktow, komentarzami do merge requestow i analiza deterministyczna.',
      bn: 'GitLab CI-তে AigisCode ইন্টিগ্রেট করুন। আর্টিফ্যাক্ট হ্যান্ডলিং, merge request বিশ্লেষণ এবং কনফিগারযোগ্য থ্রেশহোল্ড সহ প্রস্তুত .gitlab-ci.yml টেমপ্লেট।',
    },
    content: {
      en: `<h2 id="gitlab-ci-overview">AigisCode in GitLab CI</h2>
<p>GitLab CI/CD provides a powerful pipeline system that integrates tightly with merge requests, artifacts, and environments. AigisCode fits naturally into GitLab pipelines as a code quality job that runs alongside your existing test and lint stages.</p>
<p>The integration uses a standard <code>.gitlab-ci.yml</code> job definition with Python, runs the analysis, and uploads the JSON report as a GitLab artifact. You can configure it to run on merge requests only, on all pushes, or on a schedule.</p>

<h2 id="gitlab-ci-template">Pipeline Template</h2>
<p>Add the following to your <code>.gitlab-ci.yml</code>:</p>
<pre><code>stages:
  - test
  - quality

aigiscode:
  stage: quality
  image: rust:1.77-slim
  script:
    - cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
    - aigiscode analyze .
  artifacts:
    paths:
      - .aigiscode/deterministic-analysis.json
      - .aigiscode/reports/
    expire_in: 30 days
    when: always
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH</code></pre>

<h2 id="merge-request-integration">Merge Request Integration</h2>
<p>GitLab supports code quality reports natively. You can extend the job to generate a GitLab-compatible code quality report alongside the standard AigisCode JSON output. This surfaces findings directly in the merge request diff view:</p>
<pre><code>aigiscode:
  stage: quality
  image: rust:1.77-slim
  script:
    - cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
    - aigiscode analyze .
    - |
      python3 -c "
      import json
      report = json.load(open('.aigiscode/deterministic-analysis.json'))
      findings = []
      for item in report.get('dead_code', []):
          findings.append({
              'description': f'Dead code: {item.get(\"type\", \"unknown\")} in {item.get(\"name\", \"\")}',
              'fingerprint': str(hash(json.dumps(item, sort_keys=True))),
              'severity': 'minor',
              'location': {
                  'path': item.get('file', ''),
                  'lines': {'begin': item.get('line', 1)}
              }
          })
      json.dump(findings, open('gl-code-quality-report.json', 'w'))
      "
  artifacts:
    paths:
      - .aigiscode/deterministic-analysis.json
    reports:
      codequality: gl-code-quality-report.json
    expire_in: 30 days
    when: always</code></pre>

<h2 id="scheduled-analysis">Scheduled Full Analysis</h2>
<p>For the full AI-assisted analysis (which requires API keys), create a separate scheduled pipeline that runs daily or weekly:</p>
<pre><code>aigiscode-full:
  stage: quality
  image: rust:1.77-slim
  variables:
    OPENAI_API_KEY: $OPENAI_API_KEY
  script:
    - cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
    - aigiscode analyze .
  artifacts:
    paths:
      - .aigiscode/deterministic-analysis.json
      - .aigiscode/reports/
    expire_in: 90 days
  rules:
    - if: $CI_PIPELINE_SOURCE == "schedule"</code></pre>
<p>Store your API keys as CI/CD variables in the GitLab project settings. This keeps credentials out of your repository while enabling the full analysis on a regular cadence.</p>

<h2 id="parallel-jobs">Running in Parallel</h2>
<p>AigisCode analysis typically takes 10-60 seconds depending on project size. Place it in a separate stage or use <code>needs: []</code> to run it in parallel with your test suite, so it does not block your pipeline:</p>
<pre><code>aigiscode:
  stage: quality
  needs: []   # Run immediately, don't wait for test stage
  # ... rest of config</code></pre>`,
      cs: `<h2 id="gitlab-ci-overview">AigisCode v GitLab CI</h2>
<p>GitLab CI/CD poskytuje výkonný pipeline systém s těsnou integrací na merge requesty, artefakty a prostředí. AigisCode se přirozeně hodí jako job pro kvalitu kódu vedle existujících testovacích a lintovacích fází.</p>

<h2 id="gitlab-ci-template">Šablona pipeline</h2>
<p>Přidejte do <code>.gitlab-ci.yml</code> job s obrazem <code>rust:1.77-slim</code>, nainstalujte AigisCode a spusťte analýzu. Artefakty nastavte na <code>.aigiscode/deterministic-analysis.json</code> s expirací 30 dní.</p>

<h2 id="merge-request-integration">Integrace s merge requesty</h2>
<p>GitLab nativně podporuje reporty kvality kódu. Rozšiřte job o generování GitLab-kompatibilního reportu, který zobrazí nálezy přímo v diff zobrazení merge requestu.</p>

<h2 id="scheduled-analysis">Plánovaná plná analýza</h2>
<p>Pro plnou AI-asistovanou analýzu vytvořte samostatný plánovaný pipeline s API klíči uloženými jako CI/CD proměnné.</p>

<h2 id="parallel-jobs">Paralelní běh</h2>
<p>Použijte <code>needs: []</code> pro okamžité spuštění AigisCode paralelně s testy, aby neblokoval pipeline.</p>`,
      fr: `<h2 id="gitlab-ci-overview">AigisCode dans GitLab CI</h2>
<p>GitLab CI/CD fournit un systeme de pipeline puissant avec une integration etroite aux merge requests, artefacts et environnements. AigisCode s'integre naturellement comme job de qualite de code.</p>

<h2 id="gitlab-ci-template">Template de pipeline</h2>
<p>Ajoutez un job dans <code>.gitlab-ci.yml</code> avec l'image <code>rust:1.77-slim</code>, installez AigisCode et executez l'analyse. Configurez les artefacts sur <code>.aigiscode/deterministic-analysis.json</code>.</p>

<h2 id="merge-request-integration">Integration merge request</h2>
<p>GitLab supporte nativement les rapports de qualite de code. Etendez le job pour generer un rapport compatible GitLab qui affiche les resultats dans la vue diff de la merge request.</p>

<h2 id="scheduled-analysis">Analyse complete planifiee</h2>
<p>Pour l'analyse complete assistee par IA, creez un pipeline planifie separe avec les cles API stockees comme variables CI/CD.</p>

<h2 id="parallel-jobs">Execution parallele</h2>
<p>Utilisez <code>needs: []</code> pour executer AigisCode immediatement en parallele avec les tests.</p>`,
      es: `<h2 id="gitlab-ci-overview">AigisCode en GitLab CI</h2>
<p>GitLab CI/CD proporciona un potente sistema de pipeline con integracion estrecha a merge requests, artefactos y entornos. AigisCode se integra naturalmente como un job de calidad de codigo.</p>

<h2 id="gitlab-ci-template">Template de pipeline</h2>
<p>Agregue un job en <code>.gitlab-ci.yml</code> con la imagen <code>rust:1.77-slim</code>, instale AigisCode y ejecute el analisis. Configure artefactos en <code>.aigiscode/deterministic-analysis.json</code>.</p>

<h2 id="merge-request-integration">Integracion con merge request</h2>
<p>GitLab soporta nativamente reportes de calidad de codigo. Extienda el job para generar un reporte compatible con GitLab que muestre hallazgos en la vista diff del merge request.</p>

<h2 id="scheduled-analysis">Analisis completo programado</h2>
<p>Para el analisis completo asistido por IA, cree un pipeline programado separado con claves API almacenadas como variables CI/CD.</p>

<h2 id="parallel-jobs">Ejecucion paralela</h2>
<p>Use <code>needs: []</code> para ejecutar AigisCode inmediatamente en paralelo con las pruebas.</p>`,
      zh: `<h2 id="gitlab-ci-overview">GitLab CI 中的 AigisCode</h2>
<p>GitLab CI/CD 提供强大的流水线系统，与合并请求、产物和环境紧密集成。AigisCode 作为代码质量作业自然融入 GitLab 流水线。</p>

<h2 id="gitlab-ci-template">流水线模板</h2>
<p>在 <code>.gitlab-ci.yml</code> 中添加使用 <code>rust:1.77-slim</code> 镜像的作业，安装 AigisCode 并运行分析。将产物配置为 <code>.aigiscode/deterministic-analysis.json</code>。</p>

<h2 id="merge-request-integration">合并请求集成</h2>
<p>GitLab 原生支持代码质量报告。扩展作业以生成兼容 GitLab 的报告，在合并请求的 diff 视图中直接显示发现结果。</p>

<h2 id="scheduled-analysis">计划完整分析</h2>
<p>对于完整的 AI 辅助分析，创建单独的计划流水线，API 密钥存储为 CI/CD 变量。</p>

<h2 id="parallel-jobs">并行运行</h2>
<p>使用 <code>needs: []</code> 使 AigisCode 与测试并行立即运行。</p>`,
      hi: `<h2 id="gitlab-ci-overview">GitLab CI में AigisCode</h2>
<p>GitLab CI/CD मर्ज रिक्वेस्ट, आर्टिफैक्ट और एनवायरनमेंट के साथ टाइट इंटीग्रेशन वाला शक्तिशाली पाइपलाइन सिस्टम प्रदान करता है। AigisCode कोड क्वालिटी जॉब के रूप में स्वाभाविक रूप से फिट होता है।</p>

<h2 id="gitlab-ci-template">पाइपलाइन टेम्प्लेट</h2>
<p><code>.gitlab-ci.yml</code> में <code>rust:1.77-slim</code> इमेज के साथ जॉब जोड़ें, AigisCode इंस्टॉल करें और एनालिसिस चलाएं। आर्टिफैक्ट <code>.aigiscode/deterministic-analysis.json</code> पर कॉन्फ़िगर करें।</p>

<h2 id="merge-request-integration">मर्ज रिक्वेस्ट इंटीग्रेशन</h2>
<p>GitLab कोड क्वालिटी रिपोर्ट को नेटिवली सपोर्ट करता है। मर्ज रिक्वेस्ट के diff व्यू में सीधे फाइंडिंग दिखाने के लिए जॉब को एक्सटेंड करें।</p>

<h2 id="scheduled-analysis">शेड्यूल्ड पूर्ण विश्लेषण</h2>
<p>पूर्ण AI-सहायित विश्लेषण के लिए, CI/CD वेरिएबल के रूप में स्टोर API कुंजियों के साथ अलग शेड्यूल्ड पाइपलाइन बनाएं।</p>

<h2 id="parallel-jobs">पैरेलल एक्ज़ीक्यूशन</h2>
<p>टेस्ट के साथ समानांतर रूप से AigisCode को तुरंत चलाने के लिए <code>needs: []</code> का उपयोग करें।</p>`,
      pt: `<h2 id="gitlab-ci-overview">AigisCode no GitLab CI</h2>
<p>O GitLab CI/CD fornece um sistema de pipeline poderoso com integracao estreita a merge requests, artefatos e ambientes. O AigisCode se integra naturalmente como um job de qualidade de codigo.</p>

<h2 id="gitlab-ci-template">Template de pipeline</h2>
<p>Adicione um job no <code>.gitlab-ci.yml</code> com a imagem <code>rust:1.77-slim</code>, instale o AigisCode e execute a analise. Configure artefatos em <code>.aigiscode/deterministic-analysis.json</code>.</p>

<h2 id="merge-request-integration">Integracao com merge request</h2>
<p>O GitLab suporta nativamente relatorios de qualidade de codigo. Estenda o job para gerar um relatorio compativel com GitLab que exibe descobertas na visualizacao diff do merge request.</p>

<h2 id="scheduled-analysis">Analise completa agendada</h2>
<p>Para analise completa assistida por IA, crie um pipeline agendado separado com chaves API armazenadas como variaveis CI/CD.</p>

<h2 id="parallel-jobs">Execucao paralela</h2>
<p>Use <code>needs: []</code> para executar o AigisCode imediatamente em paralelo com os testes.</p>`,
      ar: `<h2 id="gitlab-ci-overview">AigisCode في GitLab CI</h2>
<p>يوفر GitLab CI/CD نظام خطوط أنابيب قوي مع تكامل وثيق مع طلبات الدمج والمخرجات والبيئات. يتكامل AigisCode بشكل طبيعي كمهمة جودة شيفرة.</p>

<h2 id="gitlab-ci-template">قالب خط الأنابيب</h2>
<p>أضف مهمة في <code>.gitlab-ci.yml</code> مع صورة <code>rust:1.77-slim</code> وثبّت AigisCode وشغّل التحليل. كوّن المخرجات في <code>.aigiscode/deterministic-analysis.json</code>.</p>

<h2 id="merge-request-integration">تكامل طلبات الدمج</h2>
<p>يدعم GitLab أصلاً تقارير جودة الشيفرة. وسّع المهمة لتوليد تقرير متوافق مع GitLab يعرض النتائج في عرض الفروقات لطلب الدمج.</p>

<h2 id="scheduled-analysis">تحليل مجدول كامل</h2>
<p>للتحليل الكامل بمساعدة الذكاء الاصطناعي أنشئ خط أنابيب مجدول منفصل مع مفاتيح API مخزنة كمتغيرات CI/CD.</p>

<h2 id="parallel-jobs">التشغيل المتوازي</h2>
<p>استخدم <code>needs: []</code> لتشغيل AigisCode فوراً بالتوازي مع الاختبارات.</p>`,
      pl: `<h2 id="gitlab-ci">GitLab CI</h2>
<p>Szablon GitLab CI/CD z konfiguracja zadan, obsluga artefaktow i komentarzami do merge requestow.</p>`,
    },
    features: [
      'Ready-to-use .gitlab-ci.yml',
      'GitLab code quality reports',
      'Merge request integration',
      'Artifact expiration control',
      'Scheduled full analysis',
      'Parallel job support',
    ],
    codeExample: `# .gitlab-ci.yml
aigiscode:
  stage: quality
  image: rust:1.77-slim
  script:
    - cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode
    - aigiscode analyze .
  artifacts:
    paths:
      - .aigiscode/deterministic-analysis.json
    expire_in: 30 days
    when: always
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH`,
    relatedSlugs: ['ci-cd-pipelines', 'github-actions', 'pre-commit-hooks'],
  },

  /* ---- 4. AI Coding Agents ---- */
  {
    slug: 'ai-coding-agents',
    icon: 'Robot',
    category: 'AI',
    status: 'available',
    title: {
      en: 'AI Coding Agents',
      cs: 'AI Kódovací Agenti',
      fr: 'Agents de Codage IA',
      es: 'Agentes de Codificación IA',
      zh: 'AI 编码代理',
      hi: 'AI कोडिंग एजेंट',
      pt: 'Agentes de Codificação IA',
      ar: 'وكلاء البرمجة بالذكاء الاصطناعي',
      pl: 'Agenci AI do kodowania',
      bn: 'AI কোডিং এজেন্ট',
    },
    shortDescription: {
      en: 'AigisCode produces machine-readable JSON output designed for AI agent consumption. Follow the recommended 8-step workflow to triage, fix, and tune findings programmatically.',
      cs: 'AigisCode generuje strojově čitelný JSON výstup navržený pro spotřebu AI agenty. Postupujte podle doporučeného 8-krokového workflow pro třídění, opravu a ladění nálezů programově.',
      fr: 'AigisCode produit une sortie JSON lisible par machine, conçue pour les agents IA. Suivez le workflow recommandé en 8 étapes pour trier, corriger et affiner les résultats de manière programmatique.',
      es: 'AigisCode produce una salida JSON legible por máquina diseñada para el consumo de agentes IA. Siga el workflow recomendado de 8 pasos para clasificar, corregir y ajustar hallazgos programáticamente.',
      zh: 'AigisCode 生成专为 AI 代理设计的机器可读 JSON 输出。按照推荐的 8 步工作流程，以编程方式分类、修复和调优发现结果。',
      hi: 'AigisCode AI एजेंट उपभोग के लिए डिज़ाइन किया गया मशीन-रीडेबल JSON आउटपुट उत्पन्न करता है। प्रोग्रामेटिक रूप से खोजों को वर्गीकृत, ठीक और ट्यून करने के लिए अनुशंसित 8-चरण वर्कफ़्लो का पालन करें।',
      pt: 'O AigisCode produz saída JSON legível por máquina, projetada para consumo por agentes IA. Siga o workflow recomendado de 8 etapas para triagem, correção e ajuste programático dos achados.',
      ar: 'ينتج AigisCode مخرجات JSON قابلة للقراءة آلياً مصممة لاستهلاك وكلاء الذكاء الاصطناعي. اتبع سير العمل الموصى به من ٨ خطوات لفرز النتائج وإصلاحها وضبطها برمجياً.',
      pl: 'AigisCode generuje czytelne maszynowo wyjscie JSON zaprojektowane do konsumpcji przez agentow AI. Postepuj wedlug zalecanego 8-etapowego workflow agenta: odczytaj znaleziska, zaplanuj poprawki, wdroz zmiany, ponownie uruchom analize, zweryfikuj poprawki, zatwierdz.',
      bn: 'AigisCode AI এজেন্ট ব্যবহারের জন্য ডিজাইন করা মেশিন-রিডেবল JSON আউটপুট তৈরি করে। প্রোগ্রামেটিকভাবে ফলাফল ট্রায়াজ, ঠিক এবং টিউন করতে প্রস্তাবিত ৮-ধাপ ওয়ার্কফ্লো অনুসরণ করুন।',
    },
    metaDescription: {
      en: 'Use AigisCode with AI coding agents. Machine-readable JSON output, 8-step agent workflow, structured findings for automated code improvement and architectural health monitoring.',
      cs: 'Používejte AigisCode s AI kódovacími agenty. Strojově čitelný JSON výstup, 8-krokový workflow pro agenty, strukturované nálezy pro automatizované vylepšování kódu a sledování architektonického zdraví.',
      fr: 'Utilisez AigisCode avec les agents de codage IA. Sortie JSON lisible par machine, workflow agent en 8 étapes, résultats structurés pour l\'amélioration automatisée du code et la surveillance de la santé architecturale.',
      es: 'Use AigisCode con agentes de codificación IA. Salida JSON legible por máquina, workflow de agente de 8 pasos, hallazgos estructurados para mejora automatizada del código y monitoreo de salud arquitectónica.',
      zh: '将 AigisCode 与 AI 编码代理配合使用。机器可读的 JSON 输出、8 步代理工作流程、用于自动化代码改进和架构健康监控的结构化发现结果。',
      hi: 'AI कोडिंग एजेंट के साथ AigisCode का उपयोग करें। मशीन-रीडेबल JSON आउटपुट, 8-चरण एजेंट वर्कफ़्लो, स्वचालित कोड सुधार और आर्किटेक्चरल हेल्थ मॉनिटरिंग के लिए संरचित खोज।',
      pt: 'Use o AigisCode com agentes de codificação IA. Saída JSON legível por máquina, workflow de agente em 8 etapas, achados estruturados para melhoria automatizada de código e monitoramento de saúde arquitetural.',
      ar: 'استخدم AigisCode مع وكلاء البرمجة بالذكاء الاصطناعي. مخرجات JSON قابلة للقراءة آلياً وسير عمل من ٨ خطوات للوكيل ونتائج منظمة لتحسين الشيفرة الآلي ومراقبة الصحة المعمارية.',
      pl: 'Uzyj AigisCode z agentami AI do kodowania. Czytelne maszynowo wyjscie JSON, 8-etapowy workflow agenta, ustrukturyzowane znaleziska dla Claude Code, Codex i innych narzedzi AI do programowania.',
      bn: 'AI কোডিং এজেন্টদের সাথে AigisCode ব্যবহার করুন। মেশিন-রিডেবল JSON আউটপুট, ৮-ধাপ এজেন্ট ওয়ার্কফ্লো, স্বয়ংক্রিয় কোড উন্নতি এবং আর্কিটেকচারাল স্বাস্থ্য পর্যবেক্ষণের জন্য কাঠামোবদ্ধ ফলাফল।',
    },
    content: {
      en: `<h2 id="ai-agent-integration">Designed for AI Agents</h2>
<p>AigisCode was built with AI coding agents in mind. The primary output is a structured JSON report at <code>.aigiscode/deterministic-analysis.json</code> that agents can parse, triage, and act on without any human intervention. Every finding includes a file path, line number (where applicable), confidence level, and category — the exact structure agents need to make informed decisions.</p>
<p>Whether you are using Claude Code, GitHub Copilot Workspace, Cursor, Codex, Devin, or a custom agent built on the OpenAI or Anthropic APIs, AigisCode provides the structured input your agent needs to understand architectural problems and generate targeted fixes.</p>

<h2 id="json-output-structure">JSON Output Structure</h2>
<p>The report JSON has a consistent top-level structure that agents can rely on:</p>
<pre><code>{
  "summary": {
    "total_files": 342,
    "circular_dependencies": 3,
    "dead_code": 47,
    "hardwired_values": 12,
    "god_classes": 2,
    "bottleneck_files": 5,
    "orphan_files": 8,
    "detector_coverage": { "php": 342, "python": 0 }
  },
  "graph_analysis": {
    "strong_circular_dependencies": [...],
    "circular_dependencies": [...],
    "god_classes": [...],
    "bottleneck_files": [...],
    "orphan_files": [...]
  },
  "dead_code": [...],
  "hardwiring": [...],
  "extensions": {
    "contract_inventory": {
      "routes": [...],
      "hooks": [...],
      "env_vars": [...],
      "config_keys": [...]
    }
  }
}</code></pre>
<p>Each finding array contains objects with <code>file</code>, <code>line</code>, <code>type</code>, <code>name</code>, <code>confidence</code>, and <code>details</code> fields. Agents can filter by confidence level to prioritize high-signal findings.</p>

<h2 id="recommended-workflow">Recommended 8-Step Agent Workflow</h2>
<p>Follow this workflow to get the most value from AigisCode in an automated agent pipeline:</p>
<ol>
<li><strong>Run analysis</strong> — Execute <code>aigiscode analyze /repo</code> to generate the baseline report.</li>
<li><strong>Parse report</strong> — Load <code>.aigiscode/deterministic-analysis.json</code> and extract structured findings.</li>
<li><strong>Triage by severity</strong> — Sort findings by confidence level. Start with <code>high</code> confidence findings that are most likely true positives.</li>
<li><strong>Sample and classify</strong> — For each finding, read the referenced code and classify it as true positive, false positive, or uncertain.</li>
<li><strong>Apply fixes</strong> — For confirmed true positives, generate and apply targeted fixes. For circular dependencies, extract shared types. For dead code, remove the unused symbols.</li>
<li><strong>Add exclusion rules</strong> — For confirmed false positives, add entries to <code>.aigiscode/rules.json</code> so the same pattern is not flagged again.</li>
<li><strong>Encode policy patterns</strong> — If multiple false positives share a common pattern (e.g., all test helpers), encode the pattern in <code>.aigiscode/policy.json</code>.</li>
<li><strong>Re-evaluate</strong> — Run <code>aigiscode analyze /repo</code> for fast re-evaluation after policy changes to confirm the fix or suppression worked.</li>
</ol>

<h2 id="parsing-specific-findings">Parsing Specific Finding Types</h2>
<p>Agents should use <code>jq</code> or equivalent JSON processing to extract specific finding types:</p>
<pre><code># Strong circular dependencies (highest architectural priority)
jq '.graph_analysis.strong_circular_dependencies' report.json

# Unused imports (quick wins for cleanup)
jq '.dead_code[] | select(.type == "unused_import")' report.json

# Hardcoded URLs and IPs (security-adjacent findings)
jq '.hardwiring[] | select(.type == "hardcoded_network")' report.json

# God classes (refactoring candidates)
jq '.graph_analysis.god_classes' report.json

# Files with no inbound dependencies (potential orphans)
jq '.graph_analysis.orphan_files' report.json</code></pre>

<h2 id="policy-customization">Policy Customization for Agents</h2>
<p>Agents can create and modify <code>.aigiscode/policy.json</code> to adapt AigisCode behavior to the specific project. This is the preferred way to reduce false positives — change policy instead of changing agent behavior:</p>
<pre><code>{
  "graph": {
    "js_import_aliases": { "@/": "src/" },
    "orphan_entry_patterns": ["src/bootstrap/**/*.ts"]
  },
  "dead_code": {
    "abandoned_entry_patterns": ["/Contracts/"]
  },
  "hardwiring": {
    "repeated_literal_min_occurrences": 4,
    "skip_path_patterns": ["app/Console/*"]
  }
}</code></pre>
<p>This approach ensures that suppression decisions are transparent, versioned in the repository, and apply consistently across all future runs.</p>`,
      cs: `<h2 id="ai-agent-integration">Navrženo pro AI agenty</h2>
<p>AigisCode byl postaven s ohledem na AI kódovací agenty. Primární výstup je strukturovaný JSON report na <code>.aigiscode/deterministic-analysis.json</code>, který agenti mohou parsovat, třídit a jednat na základě něj bez lidského zásahu. Každý nález obsahuje cestu k souboru, číslo řádku, úroveň jistoty a kategorii.</p>

<h2 id="json-output-structure">Struktura JSON výstupu</h2>
<p>Report JSON má konzistentní top-level strukturu s klíči: <code>summary</code>, <code>graph_analysis</code>, <code>dead_code</code>, <code>hardwiring</code> a <code>extensions</code>. Agenti mohou filtrovat podle úrovně jistoty.</p>

<h2 id="recommended-workflow">Doporučený 8-krokový workflow</h2>
<ol>
<li>Spusťte analýzu (<code>aigiscode analyze /repo</code>)</li>
<li>Parsujte report</li>
<li>Třiďte podle závažnosti</li>
<li>Vzorkujte a klasifikujte</li>
<li>Aplikujte opravy</li>
<li>Přidejte pravidla vyloučení do <code>.aigiscode/rules.json</code></li>
<li>Zakódujte vzory do <code>.aigiscode/policy.json</code></li>
<li>Znovu vyhodnoťte pomocí <code>aigiscode analyze /repo</code></li>
</ol>

<h2 id="policy-customization">Přizpůsobení politiky</h2>
<p>Agenti mohou vytvářet a upravovat <code>.aigiscode/policy.json</code> pro přizpůsobení chování AigisCode konkrétnímu projektu. Měňte politiku místo chování agenta.</p>`,
      fr: `<h2 id="ai-agent-integration">Concu pour les agents IA</h2>
<p>AigisCode a ete concu pour les agents de codage IA. La sortie principale est un rapport JSON structure a <code>.aigiscode/deterministic-analysis.json</code> que les agents peuvent analyser et exploiter sans intervention humaine. Chaque resultat inclut le chemin du fichier, le numero de ligne, le niveau de confiance et la categorie.</p>

<h2 id="json-output-structure">Structure de sortie JSON</h2>
<p>Le rapport JSON a une structure de niveau superieur coherente avec les cles : <code>summary</code>, <code>graph_analysis</code>, <code>dead_code</code>, <code>hardwiring</code> et <code>extensions</code>. Les agents peuvent filtrer par niveau de confiance.</p>

<h2 id="recommended-workflow">Workflow agent en 8 etapes</h2>
<ol>
<li>Executer l'analyse (<code>aigiscode analyze /repo</code>)</li>
<li>Analyser le rapport</li>
<li>Trier par severite</li>
<li>Echantillonner et classifier</li>
<li>Appliquer les corrections</li>
<li>Ajouter des regles d'exclusion dans <code>.aigiscode/rules.json</code></li>
<li>Encoder les patterns dans <code>.aigiscode/policy.json</code></li>
<li>Re-evaluer avec <code>aigiscode analyze /repo</code></li>
</ol>

<h2 id="policy-customization">Personnalisation de la politique</h2>
<p>Les agents peuvent creer et modifier <code>.aigiscode/policy.json</code> pour adapter le comportement d'AigisCode. Changez la politique plutot que le comportement de l'agent.</p>`,
      es: `<h2 id="ai-agent-integration">Disenado para agentes IA</h2>
<p>AigisCode fue construido pensando en agentes de codificacion IA. La salida principal es un reporte JSON estructurado en <code>.aigiscode/deterministic-analysis.json</code> que los agentes pueden analizar y actuar sin intervencion humana. Cada hallazgo incluye ruta de archivo, numero de linea, nivel de confianza y categoria.</p>

<h2 id="json-output-structure">Estructura de salida JSON</h2>
<p>El reporte JSON tiene una estructura de nivel superior consistente con claves: <code>summary</code>, <code>graph_analysis</code>, <code>dead_code</code>, <code>hardwiring</code> y <code>extensions</code>. Los agentes pueden filtrar por nivel de confianza.</p>

<h2 id="recommended-workflow">Workflow de agente en 8 pasos</h2>
<ol>
<li>Ejecutar analisis (<code>aigiscode analyze /repo</code>)</li>
<li>Parsear reporte</li>
<li>Clasificar por severidad</li>
<li>Muestrear y clasificar</li>
<li>Aplicar correcciones</li>
<li>Agregar reglas de exclusion en <code>.aigiscode/rules.json</code></li>
<li>Codificar patrones en <code>.aigiscode/policy.json</code></li>
<li>Re-evaluar con <code>aigiscode analyze /repo</code></li>
</ol>

<h2 id="policy-customization">Personalizacion de politicas</h2>
<p>Los agentes pueden crear y modificar <code>.aigiscode/policy.json</code> para adaptar el comportamiento de AigisCode. Cambie la politica en lugar del comportamiento del agente.</p>`,
      zh: `<h2 id="ai-agent-integration">为 AI 代理设计</h2>
<p>AigisCode 专为 AI 编码代理构建。主要输出是 <code>.aigiscode/deterministic-analysis.json</code> 中的结构化 JSON 报告，代理可以无需人工干预即可解析、分类和执行操作。每个发现包含文件路径、行号、置信度和类别。</p>

<h2 id="json-output-structure">JSON 输出结构</h2>
<p>报告 JSON 具有一致的顶层结构，包含键：<code>summary</code>、<code>graph_analysis</code>、<code>dead_code</code>、<code>hardwiring</code> 和 <code>extensions</code>。代理可以按置信度级别过滤。</p>

<h2 id="recommended-workflow">推荐的 8 步代理工作流</h2>
<ol>
<li>运行分析（<code>aigiscode analyze /repo</code>）</li>
<li>解析报告</li>
<li>按严重性分类</li>
<li>抽样并分类</li>
<li>应用修复</li>
<li>在 <code>.aigiscode/rules.json</code> 中添加排除规则</li>
<li>在 <code>.aigiscode/policy.json</code> 中编码模式</li>
<li>使用 <code>aigiscode analyze /repo</code> 重新评估</li>
</ol>

<h2 id="policy-customization">代理策略自定义</h2>
<p>代理可以创建和修改 <code>.aigiscode/policy.json</code> 来调整 AigisCode 行为。更改策略而非代理行为。</p>`,
      hi: `<h2 id="ai-agent-integration">AI एजेंट के लिए डिज़ाइन किया गया</h2>
<p>AigisCode AI कोडिंग एजेंट को ध्यान में रखकर बनाया गया है। प्राथमिक आउटपुट <code>.aigiscode/deterministic-analysis.json</code> पर एक स्ट्रक्चर्ड JSON रिपोर्ट है जिसे एजेंट बिना मानवीय हस्तक्षेप के पार्स, ट्राइएज और एक्ट कर सकते हैं। हर फाइंडिंग में फ़ाइल पथ, लाइन नंबर, कॉन्फ़िडेंस लेवल और कैटेगरी शामिल है।</p>

<h2 id="json-output-structure">JSON आउटपुट स्ट्रक्चर</h2>
<p>रिपोर्ट JSON में कंसिस्टेंट टॉप-लेवल स्ट्रक्चर है: <code>summary</code>, <code>graph_analysis</code>, <code>dead_code</code>, <code>hardwiring</code> और <code>extensions</code>। एजेंट कॉन्फ़िडेंस लेवल से फ़िल्टर कर सकते हैं।</p>

<h2 id="recommended-workflow">अनुशंसित 8-चरण एजेंट वर्कफ़्लो</h2>
<ol>
<li>एनालिसिस चलाएं (<code>aigiscode analyze /repo</code>)</li>
<li>रिपोर्ट पार्स करें</li>
<li>गंभीरता से वर्गीकृत करें</li>
<li>सैंपल और क्लासिफाई करें</li>
<li>फ़िक्स लागू करें</li>
<li><code>.aigiscode/rules.json</code> में एक्सक्लूज़न रूल जोड़ें</li>
<li><code>.aigiscode/policy.json</code> में पैटर्न एनकोड करें</li>
<li><code>aigiscode analyze /repo</code> से री-इवैल्यूएट करें</li>
</ol>

<h2 id="policy-customization">एजेंट के लिए पॉलिसी कस्टमाइज़ेशन</h2>
<p>एजेंट AigisCode के व्यवहार को अनुकूलित करने के लिए <code>.aigiscode/policy.json</code> बना और संशोधित कर सकते हैं। एजेंट व्यवहार बदलने के बजाय पॉलिसी बदलें।</p>`,
      pt: `<h2 id="ai-agent-integration">Projetado para agentes IA</h2>
<p>O AigisCode foi construido pensando em agentes de codificacao IA. A saida principal e um relatorio JSON estruturado em <code>.aigiscode/deterministic-analysis.json</code> que os agentes podem analisar e agir sem intervencao humana. Cada descoberta inclui caminho do arquivo, numero da linha, nivel de confianca e categoria.</p>

<h2 id="json-output-structure">Estrutura de saida JSON</h2>
<p>O relatorio JSON tem uma estrutura de nivel superior consistente com chaves: <code>summary</code>, <code>graph_analysis</code>, <code>dead_code</code>, <code>hardwiring</code> e <code>extensions</code>. Os agentes podem filtrar por nivel de confianca.</p>

<h2 id="recommended-workflow">Workflow de agente em 8 etapas</h2>
<ol>
<li>Executar analise (<code>aigiscode analyze /repo</code>)</li>
<li>Analisar relatorio</li>
<li>Classificar por severidade</li>
<li>Amostrar e classificar</li>
<li>Aplicar correcoes</li>
<li>Adicionar regras de exclusao em <code>.aigiscode/rules.json</code></li>
<li>Codificar padroes em <code>.aigiscode/policy.json</code></li>
<li>Reavaliar com <code>aigiscode analyze /repo</code></li>
</ol>

<h2 id="policy-customization">Personalizacao de politicas</h2>
<p>Os agentes podem criar e modificar <code>.aigiscode/policy.json</code> para adaptar o comportamento do AigisCode. Mude a politica em vez do comportamento do agente.</p>`,
      ar: `<h2 id="designed-for-agents">مصمم لوكلاء الذكاء الاصطناعي</h2>
<p>يحتاج وكلاء البرمجة بالذكاء الاصطناعي مثل Claude Code وCodex بيانات منظمة عن صحة قاعدة الشيفرة. ينتج AigisCode مخرجات JSON قابلة للقراءة آلياً يمكن للوكلاء تحليلها وفرزها والتصرف بناءً عليها.</p>

<h2 id="agent-workflow">سير عمل الوكيل الموصى به من ٨ خطوات</h2>
<ol>
<li><strong>التحليل</strong> — شغّل <code>aigiscode analyze .</code>.</li>
<li><strong>التحليل النحوي</strong> — حلّل تقرير JSON واستخرج النتائج حسب النوع.</li>
<li><strong>الفرز</strong> — رتّب النتائج حسب الشدة والتأثير.</li>
<li><strong>الإصلاح</strong> — طبّق إصلاحات آلية للنتائج عالية الثقة.</li>
<li><strong>التحقق</strong> — أعد تشغيل التحليل للتأكد من حل النتائج.</li>
<li><strong>الضبط</strong> — حدّث <code>policy.json</code> للسلوك المحدد بالمشروع.</li>
<li><strong>الإبلاغ</strong> — أنشئ ملخصاً بالتغييرات والنتائج المتبقية.</li>
<li><strong>التكرار</strong> — كرر للنتائج الأقل أولوية.</li>
</ol>

<h2 id="policy-integration">تكامل السياسات</h2>
<p>يستخدم الوكلاء <code>policy.json</code> لفهم قواعد المشروع وأنماط الاستبعاد وعتبات الشدة. يتيح هذا اتخاذ قرارات واعية بالسياق.</p>

<h2 id="future">مستقبل الصيانة المدفوعة بالذكاء الاصطناعي</h2>
<p>مع ازدياد قدرة الوكلاء ستصبح حلقة التحليل-الفرز-الإصلاح مستقلة بالكامل. صُمم AigisCode من الأساس ليكون طبقة الاستشعار لصيانة الشيفرة المدفوعة بالذكاء الاصطناعي.</p>`,
      pl: `<h2 id="ai-agents">Agenci AI do kodowania</h2>
<p>AigisCode generuje czytelne maszynowo wyjscie JSON do konsumpcji przez agentow AI. Zalecany 8-etapowy workflow agenta.</p>`,
    },
    features: [
      'Machine-readable JSON output',
      '8-step agent workflow',
      'Confidence-based triage',
      'Policy-driven suppression',
      'jq-friendly report structure',
      'Works with any AI agent',
    ],
    codeExample: `# Step 1: Generate the report
aigiscode analyze /path/to/project

# Step 2: Parse findings
cat .aigiscode/deterministic-analysis.json | jq '.summary'

# Step 3: Extract high-priority findings
cat .aigiscode/deterministic-analysis.json | jq '.graph_analysis.strong_circular_dependencies'

# Step 4: After fixes, re-evaluate
aigiscode analyze /path/to/project`,
    relatedSlugs: ['ci-cd-pipelines', 'github-actions', 'webhooks'],
  },

  /* ---- 5. IDE Plugins ---- */
  {
    slug: 'ide-plugins',
    icon: 'Kanban',
    category: 'Developer Tools',
    status: 'coming-soon',
    title: {
      en: 'IDE Plugins',
      cs: 'IDE Pluginy',
      fr: 'Plugins IDE',
      es: 'Plugins para IDE',
      zh: 'IDE 插件',
      hi: 'IDE प्लगइन',
      pt: 'Plugins para IDE',
      ar: 'إضافات بيئة التطوير',
      pl: 'Pluginy IDE',
      bn: 'IDE Plugin',
    },
    shortDescription: {
      en: 'Editor integrations for real-time architectural feedback. Get inline warnings for circular dependencies, dead code, and hardwired values as you type.',
      cs: 'Integrace do editorů pro architektonickou zpětnou vazbu v reálném čase. Získejte vložená varování o cyklických závislostech, mrtvém kódu a natvrdo zapsaných hodnotách přímo při psaní.',
      fr: 'Intégrations dans l\'éditeur pour un retour architectural en temps réel. Recevez des avertissements en ligne sur les dépendances circulaires, le code mort et les valeurs codées en dur pendant que vous tapez.',
      es: 'Integraciones en el editor para retroalimentación arquitectónica en tiempo real. Obtenga advertencias en línea sobre dependencias circulares, código muerto y valores codificados mientras escribe.',
      zh: '编辑器集成，提供实时架构反馈。在您编写代码时获取有关循环依赖、死代码和硬编码值的内联警告。',
      hi: 'रियल-टाइम आर्किटेक्चरल फ़ीडबैक के लिए एडिटर इंटीग्रेशन। टाइप करते समय सर्कुलर डिपेंडेंसी, डेड कोड और हार्डवायर्ड वैल्यू के लिए इनलाइन चेतावनियां प्राप्त करें।',
      pt: 'Integrações no editor para feedback arquitetural em tempo real. Receba avisos inline sobre dependências circulares, código morto e valores codificados enquanto digita.',
      ar: 'تكاملات المحرر للتغذية الراجعة المعمارية الفورية. احصل على تحذيرات مضمّنة للتبعيات الدائرية والشيفرة الميتة والقيم الثابتة أثناء الكتابة.',
      pl: 'Integracje z edytorami do informacji zwrotnych o architekturze w czasie rzeczywistym. Otrzymuj ostrzezenia inline o cyklicznych zaleznosciach, znaczniki martwego kodu i podswietlenie zakodowanych wartosci bezposrednio w IDE.',
      bn: 'রিয়েল-টাইম আর্কিটেকচারাল ফিডব্যাকের জন্য এডিটর ইন্টিগ্রেশন। টাইপ করার সাথে সাথে সার্কুলার ডিপেন্ডেন্সি, ডেড কোড এবং হার্ডওয়্যার্ড ভ্যালুর জন্য ইনলাইন সতর্কতা পান।',
    },
    metaDescription: {
      en: 'AigisCode IDE plugins for VS Code, JetBrains, and Neovim. Real-time architectural analysis with inline warnings for circular dependencies and dead code. Coming soon.',
      cs: 'IDE pluginy AigisCode pro VS Code, JetBrains a Neovim. Architektonická analýza v reálném čase s vloženými varováními o cyklických závislostech a mrtvém kódu. Již brzy.',
      fr: 'Plugins IDE AigisCode pour VS Code, JetBrains et Neovim. Analyse architecturale en temps réel avec avertissements en ligne pour les dépendances circulaires et le code mort. Bientôt disponible.',
      es: 'Plugins IDE de AigisCode para VS Code, JetBrains y Neovim. Análisis arquitectónico en tiempo real con advertencias en línea para dependencias circulares y código muerto. Próximamente.',
      zh: 'AigisCode IDE 插件，支持 VS Code、JetBrains 和 Neovim。实时架构分析，提供循环依赖和死代码的内联警告。即将推出。',
      hi: 'VS Code, JetBrains और Neovim के लिए AigisCode IDE प्लगइन। सर्कुलर डिपेंडेंसी और डेड कोड के लिए इनलाइन चेतावनियों के साथ रियल-टाइम आर्किटेक्चरल विश्लेषण। जल्द आ रहा है।',
      pt: 'Plugins IDE do AigisCode para VS Code, JetBrains e Neovim. Análise arquitetural em tempo real com avisos inline para dependências circulares e código morto. Em breve.',
      ar: 'إضافات AigisCode لبيئات التطوير VS Code وJetBrains وNeovim. تحليل معماري فوري مع تحذيرات مضمّنة للتبعيات الدائرية والشيفرة الميتة. قريباً.',
      pl: 'Pluginy IDE AigisCode dla VS Code, JetBrains i Neovim. Analiza architektury w czasie rzeczywistym z ostrzezeniami inline, znacznikami martwego kodu i podswietleniem zakodowanych wartosci.',
      bn: 'VS Code, JetBrains এবং Neovim-এর জন্য AigisCode IDE plugin। সার্কুলার ডিপেন্ডেন্সি এবং ডেড কোডের জন্য ইনলাইন সতর্কতা সহ রিয়েল-টাইম আর্কিটেকচারাল বিশ্লেষণ। শীঘ্রই আসছে।',
    },
    content: {
      en: `<h2 id="ide-integration-vision">Real-Time Architectural Feedback</h2>
<p>AigisCode IDE plugins will bring architectural analysis directly into your editor. Instead of waiting for CI to tell you about a circular dependency, you will see the warning inline as soon as you add the import that creates the cycle.</p>
<p>The plugin will run AigisCode in the background, watch for file changes, and incrementally update findings. This means near-instant feedback without the overhead of a full project re-analysis on every keystroke.</p>

<h2 id="planned-editors">Planned Editor Support</h2>
<p>We are developing plugins for the three most popular editor families:</p>
<ul>
<li><strong>VS Code</strong> — Extension with diagnostics panel, inline decorations, and CodeLens for quick-fix actions. Uses the VS Code Language Server Protocol for efficient communication.</li>
<li><strong>JetBrains</strong> — Plugin for IntelliJ IDEA, PhpStorm, PyCharm, and WebStorm. Integrates with JetBrains inspections and the built-in problem view.</li>
<li><strong>Neovim</strong> — LSP client integration via nvim-lsp. Provides diagnostics, code actions, and a Telescope picker for navigating findings.</li>
</ul>

<h2 id="planned-features">Planned Features</h2>
<p>The IDE plugins will support the following capabilities:</p>
<ul>
<li><strong>Inline diagnostics</strong> — Circular dependencies, dead imports, and hardwired values shown as squiggly underlines with hover tooltips explaining the issue.</li>
<li><strong>Quick-fix actions</strong> — One-click resolution for common issues like removing unused imports or extracting hardcoded values to configuration.</li>
<li><strong>Dependency graph view</strong> — Visual panel showing the import graph for the current file and highlighting any cycles it participates in.</li>
<li><strong>Status bar indicators</strong> — At-a-glance view of the project's architectural health score in the editor status bar.</li>
<li><strong>Policy-aware suppression</strong> — Right-click to add a finding to <code>.aigiscode/rules.json</code> without leaving the editor.</li>
</ul>

<h2 id="early-access">Early Access</h2>
<p>IDE plugins are currently in development. Sign up for early access to be notified when the beta is available. In the meantime, you can use AigisCode from the command line and parse the JSON report with your editor's built-in terminal or problem matcher.</p>`,
      cs: `<h2 id="ide-integration-vision">Architektonická zpětná vazba v reálném čase</h2>
<p>IDE pluginy AigisCode přinesou architektonickou analýzu přímo do vašeho editoru. Místo čekání na CI uvidíte varování inline, jakmile přidáte import, který vytvoří cyklus.</p>

<h2 id="planned-editors">Plánovaná podpora editorů</h2>
<ul>
<li><strong>VS Code</strong> — Extension s diagnostickým panelem, inline dekoracemi a CodeLens pro quick-fix akce.</li>
<li><strong>JetBrains</strong> — Plugin pro IntelliJ IDEA, PhpStorm, PyCharm a WebStorm s integrací do inspekcí.</li>
<li><strong>Neovim</strong> — Integrace LSP klienta přes nvim-lsp s diagnostikou a Telescope pickerem.</li>
</ul>

<h2 id="planned-features">Plánované funkce</h2>
<ul>
<li><strong>Inline diagnostika</strong> — Cyklické závislosti, mrtvé importy a hardwired hodnoty jako podtržení s tooltipy.</li>
<li><strong>Quick-fix akce</strong> — Oprava jedním kliknutím pro běžné problémy.</li>
<li><strong>Zobrazení grafu závislostí</strong> — Vizuální panel s importním grafem aktuálního souboru.</li>
<li><strong>Suppression s ohledem na politiku</strong> — Pravý klik pro přidání nálezu do <code>.aigiscode/rules.json</code>.</li>
</ul>

<h2 id="early-access">Předběžný přístup</h2>
<p>IDE pluginy jsou v současné době ve vývoji. Mezitím můžete používat AigisCode z příkazové řádky.</p>`,
      fr: `<h2 id="ide-integration-vision">Retour architectural en temps reel</h2>
<p>Les plugins IDE AigisCode apporteront l'analyse architecturale directement dans votre editeur. Au lieu d'attendre le CI, vous verrez l'avertissement en ligne des que vous ajoutez l'import qui cree le cycle.</p>

<h2 id="planned-editors">Support editeurs prevu</h2>
<ul>
<li><strong>VS Code</strong> — Extension avec panneau de diagnostic, decorations inline et CodeLens pour les actions de correction rapide.</li>
<li><strong>JetBrains</strong> — Plugin pour IntelliJ IDEA, PhpStorm, PyCharm et WebStorm integre aux inspections.</li>
<li><strong>Neovim</strong> — Integration client LSP via nvim-lsp avec diagnostics et selecteur Telescope.</li>
</ul>

<h2 id="planned-features">Fonctionnalites prevues</h2>
<ul>
<li><strong>Diagnostics inline</strong> — Dependances circulaires, imports morts et valeurs codees en dur comme soulignements avec infobulles.</li>
<li><strong>Actions de correction rapide</strong> — Resolution en un clic pour les problemes courants.</li>
<li><strong>Vue du graphe de dependances</strong> — Panneau visuel du graphe d'imports du fichier courant.</li>
<li><strong>Suppression respectant la politique</strong> — Clic droit pour ajouter un resultat a <code>.aigiscode/rules.json</code>.</li>
</ul>

<h2 id="early-access">Acces anticipe</h2>
<p>Les plugins IDE sont en cours de developpement. En attendant, utilisez AigisCode en ligne de commande.</p>`,
      es: `<h2 id="ide-integration-vision">Retroalimentacion arquitectonica en tiempo real</h2>
<p>Los plugins IDE de AigisCode traeran el analisis arquitectonico directamente a su editor. En lugar de esperar al CI, vera la advertencia inline tan pronto como agregue el import que crea el ciclo.</p>

<h2 id="planned-editors">Soporte de editores planificado</h2>
<ul>
<li><strong>VS Code</strong> — Extension con panel de diagnosticos, decoraciones inline y CodeLens para acciones de correccion rapida.</li>
<li><strong>JetBrains</strong> — Plugin para IntelliJ IDEA, PhpStorm, PyCharm y WebStorm integrado con inspecciones.</li>
<li><strong>Neovim</strong> — Integracion de cliente LSP via nvim-lsp con diagnosticos y selector Telescope.</li>
</ul>

<h2 id="planned-features">Caracteristicas planificadas</h2>
<ul>
<li><strong>Diagnosticos inline</strong> — Dependencias circulares, imports muertos y valores hardcoded como subrayados con tooltips.</li>
<li><strong>Acciones de correccion rapida</strong> — Resolucion con un clic para problemas comunes.</li>
<li><strong>Vista del grafo de dependencias</strong> — Panel visual del grafo de imports del archivo actual.</li>
<li><strong>Supresion con respeto a politicas</strong> — Clic derecho para agregar un hallazgo a <code>.aigiscode/rules.json</code>.</li>
</ul>

<h2 id="early-access">Acceso anticipado</h2>
<p>Los plugins IDE estan en desarrollo. Mientras tanto, use AigisCode desde la linea de comandos.</p>`,
      zh: `<h2 id="ide-integration-vision">实时架构反馈</h2>
<p>AigisCode IDE 插件将架构分析直接带入您的编辑器。无需等待 CI，当您添加创建循环的导入时，立即看到内联警告。</p>

<h2 id="planned-editors">计划的编辑器支持</h2>
<ul>
<li><strong>VS Code</strong> — 扩展，带诊断面板、内联装饰和 CodeLens 快速修复操作。</li>
<li><strong>JetBrains</strong> — IntelliJ IDEA、PhpStorm、PyCharm 和 WebStorm 插件，集成检查功能。</li>
<li><strong>Neovim</strong> — 通过 nvim-lsp 的 LSP 客户端集成，带诊断和 Telescope 选择器。</li>
</ul>

<h2 id="planned-features">计划的功能</h2>
<ul>
<li><strong>内联诊断</strong> — 循环依赖、死导入和硬编码值显示为带悬停提示的下划线。</li>
<li><strong>快速修复操作</strong> — 一键解决常见问题。</li>
<li><strong>依赖图视图</strong> — 当前文件导入图的可视面板。</li>
<li><strong>策略感知抑制</strong> — 右键将发现添加到 <code>.aigiscode/rules.json</code>。</li>
</ul>

<h2 id="early-access">抢先体验</h2>
<p>IDE 插件目前正在开发中。同时，您可以从命令行使用 AigisCode。</p>`,
      hi: `<h2 id="ide-integration-vision">रियल-टाइम आर्किटेक्चरल फ़ीडबैक</h2>
<p>AigisCode IDE प्लगइन आर्किटेक्चरल एनालिसिस सीधे आपके एडिटर में लाएंगे। CI का इंतज़ार करने के बजाय, जैसे ही आप साइकिल बनाने वाला इम्पोर्ट जोड़ते हैं, आपको इनलाइन वार्निंग दिखाई देगी।</p>

<h2 id="planned-editors">प्लान्ड एडिटर सपोर्ट</h2>
<ul>
<li><strong>VS Code</strong> — डायग्नोस्टिक्स पैनल, इनलाइन डेकोरेशन और CodeLens के साथ एक्सटेंशन।</li>
<li><strong>JetBrains</strong> — IntelliJ IDEA, PhpStorm, PyCharm और WebStorm के लिए प्लगइन।</li>
<li><strong>Neovim</strong> — nvim-lsp के माध्यम से LSP क्लाइंट इंटीग्रेशन।</li>
</ul>

<h2 id="planned-features">प्लान्ड फ़ीचर्स</h2>
<ul>
<li><strong>इनलाइन डायग्नोस्टिक्स</strong> — सर्कुलर डिपेंडेंसी, डेड इम्पोर्ट और हार्डवायर्ड वैल्यू टूलटिप के साथ अंडरलाइन के रूप में।</li>
<li><strong>क्विक-फ़िक्स एक्शन</strong> — सामान्य समस्याओं के लिए एक-क्लिक समाधान।</li>
<li><strong>डिपेंडेंसी ग्राफ़ व्यू</strong> — वर्तमान फ़ाइल के इम्पोर्ट ग्राफ़ का विज़ुअल पैनल।</li>
<li><strong>पॉलिसी-अवेयर सप्रेशन</strong> — <code>.aigiscode/rules.json</code> में जोड़ने के लिए राइट-क्लिक।</li>
</ul>

<h2 id="early-access">अर्ली एक्सेस</h2>
<p>IDE प्लगइन वर्तमान में विकास में हैं। तब तक, कमांड लाइन से AigisCode का उपयोग करें।</p>`,
      pt: `<h2 id="ide-integration-vision">Feedback arquitetural em tempo real</h2>
<p>Os plugins IDE do AigisCode traro a analise arquitetural diretamente para o seu editor. Em vez de esperar pelo CI, voce vera o aviso inline assim que adicionar o import que cria o ciclo.</p>

<h2 id="planned-editors">Suporte de editores planejado</h2>
<ul>
<li><strong>VS Code</strong> — Extensao com painel de diagnosticos, decoracoes inline e CodeLens para acoes de correcao rapida.</li>
<li><strong>JetBrains</strong> — Plugin para IntelliJ IDEA, PhpStorm, PyCharm e WebStorm integrado com inspecoes.</li>
<li><strong>Neovim</strong> — Integracao de cliente LSP via nvim-lsp com diagnosticos e seletor Telescope.</li>
</ul>

<h2 id="planned-features">Funcionalidades planejadas</h2>
<ul>
<li><strong>Diagnosticos inline</strong> — Dependencias circulares, imports mortos e valores hardcoded como sublinhados com tooltips.</li>
<li><strong>Acoes de correcao rapida</strong> — Resolucao com um clique para problemas comuns.</li>
<li><strong>Visualizacao do grafo de dependencias</strong> — Painel visual do grafo de imports do arquivo atual.</li>
<li><strong>Supressao ciente de politica</strong> — Clique direito para adicionar uma descoberta ao <code>.aigiscode/rules.json</code>.</li>
</ul>

<h2 id="early-access">Acesso antecipado</h2>
<p>Os plugins IDE estao em desenvolvimento. Enquanto isso, use o AigisCode pela linha de comando.</p>`,
      ar: `<h2 id="real-time-feedback">التغذية الراجعة المعمارية الفورية</h2>
<p>تجلب إضافات المحرر التحليل المعماري مباشرة إلى بيئة التطوير. احصل على تحذيرات مضمّنة للتبعيات الدائرية والشيفرة الميتة والقيم الثابتة أثناء الكتابة بدلاً من انتظار CI.</p>

<h2 id="planned-editors">المحررات المدعومة المخططة</h2>
<ul>
<li><strong>VS Code</strong> — إضافة كاملة مع تحذيرات مضمّنة ولوحة تبعيات وأوامر التحليل السريع.</li>
<li><strong>JetBrains</strong> — دعم IntelliJ IDEA وPhpStorm وWebStorm وPyCharm مع عمليات فحص أصلية.</li>
<li><strong>Neovim</strong> — تكامل بروتوكول LSP للتحليل المعماري الفوري في الطرفية.</li>
</ul>

<h2 id="features">الميزات المخططة</h2>
<ul>
<li><strong>تحذيرات الاستيراد</strong> — تحذير عند إضافة استيراد يخلق تبعية دائرية.</li>
<li><strong>تصور التبعيات</strong> — رسوم بيانية تفاعلية لتبعيات الملف الحالي.</li>
<li><strong>إجراءات سريعة</strong> — اقتراحات إعادة بناء بنقرة واحدة لانتهاكات الطبقات.</li>
<li><strong>عرض حالة المشروع</strong> — لوحة تعرض الصحة المعمارية الإجمالية.</li>
</ul>

<h2 id="coming-soon">قريباً</h2>
<p>إضافات المحرر قيد التطوير. تابع <code>github.com/AigisCode</code> للإعلانات.</p>`,
      pl: `<h2 id="ide-plugins">Pluginy IDE</h2>
<p>Integracje z edytorami dla informacji zwrotnych w czasie rzeczywistym. Obslugiwane: VS Code, JetBrains, Neovim.</p>`,
    },
    features: [
      'VS Code extension',
      'JetBrains plugin',
      'Neovim LSP integration',
      'Inline diagnostics',
      'Quick-fix actions',
      'Dependency graph view',
    ],
    relatedSlugs: ['pre-commit-hooks', 'ci-cd-pipelines', 'ai-coding-agents'],
  },

  /* ---- 6. Pre-commit Hooks ---- */
  {
    slug: 'pre-commit-hooks',
    icon: 'ShieldStar',
    category: 'Developer Tools',
    status: 'available',
    title: {
      en: 'Pre-commit Hooks',
      cs: 'Pre-commit Hooky',
      fr: 'Hooks Pre-commit',
      es: 'Hooks Pre-commit',
      zh: 'Pre-commit 钩子',
      hi: 'Pre-commit हुक',
      pt: 'Hooks Pre-commit',
      ar: 'خطافات ما قبل الإيداع',
      pl: 'Hooki pre-commit',
      bn: 'Pre-commit Hook',
    },
    shortDescription: {
      en: 'Block problematic code before it is committed. AigisCode integrates with the pre-commit framework to run analysis on staged files and reject commits that introduce new issues.',
      cs: 'Zablokujte problematický kód ještě před commitnutím. AigisCode se integruje s frameworkem pre-commit pro spouštění analýzy nad připravenými soubory a odmítnutí commitů, které zavádějí nové problémy.',
      fr: 'Bloquez le code problématique avant qu\'il ne soit commité. AigisCode s\'intègre au framework pre-commit pour analyser les fichiers stagés et rejeter les commits introduisant de nouveaux problèmes.',
      es: 'Bloquee código problemático antes de que sea confirmado. AigisCode se integra con el framework pre-commit para ejecutar análisis en archivos preparados y rechazar commits que introduzcan nuevos problemas.',
      zh: '在代码提交之前阻止有问题的代码。AigisCode 与 pre-commit 框架集成，对暂存文件运行分析，并拒绝引入新问题的提交。',
      hi: 'कमिट होने से पहले समस्याग्रस्त कोड को ब्लॉक करें। AigisCode स्टेज्ड फ़ाइलों पर विश्लेषण चलाने और नई समस्याएं पेश करने वाले कमिट को अस्वीकार करने के लिए pre-commit फ्रेमवर्क के साथ इंटीग्रेट होता है।',
      pt: 'Bloqueie código problemático antes de ser commitado. O AigisCode integra-se com o framework pre-commit para executar análise nos arquivos preparados e rejeitar commits que introduzam novos problemas.',
      ar: 'امنع الشيفرة الإشكالية قبل إيداعها. يتكامل AigisCode مع إطار pre-commit لتشغيل التحليل على الملفات المرحلية ورفض الإيداعات التي تُدخل مشكلات جديدة.',
      pl: 'Blokuj problematyczny kod, zanim zostanie zatwierdzony. AigisCode integruje sie z frameworkiem pre-commit, aby uruchamiac ukierunkowana analize na plikach staged, wychwytujac nowe cykliczne zaleznosci i zakodowane wartosci podczas commita.',
      bn: 'কমিট হওয়ার আগেই সমস্যাযুক্ত কোড ব্লক করুন। AigisCode স্টেজড ফাইলে বিশ্লেষণ চালাতে এবং নতুন সমস্যা প্রবর্তনকারী কমিট প্রত্যাখ্যান করতে pre-commit ফ্রেমওয়ার্কের সাথে ইন্টিগ্রেট করে।',
    },
    metaDescription: {
      en: 'Use AigisCode as a pre-commit hook to catch circular dependencies and dead code before commit. Ready-to-use .pre-commit-config.yaml configuration.',
      cs: 'Používejte AigisCode jako pre-commit hook pro zachycení cyklických závislostí a mrtvého kódu před commitnutím. Připravená konfigurace .pre-commit-config.yaml.',
      fr: 'Utilisez AigisCode comme hook pre-commit pour détecter les dépendances circulaires et le code mort avant le commit. Configuration .pre-commit-config.yaml prête à l\'emploi.',
      es: 'Use AigisCode como hook pre-commit para detectar dependencias circulares y código muerto antes del commit. Configuración .pre-commit-config.yaml lista para usar.',
      zh: '将 AigisCode 用作 pre-commit 钩子，在提交前捕获循环依赖和死代码。即用型 .pre-commit-config.yaml 配置。',
      hi: 'कमिट से पहले सर्कुलर डिपेंडेंसी और डेड कोड पकड़ने के लिए AigisCode को pre-commit हुक के रूप में उपयोग करें। तैयार .pre-commit-config.yaml कॉन्फ़िगरेशन।',
      pt: 'Use o AigisCode como hook pre-commit para capturar dependências circulares e código morto antes do commit. Configuração .pre-commit-config.yaml pronta para uso.',
      ar: 'استخدم AigisCode كخطاف ما قبل الإيداع لاكتشاف التبعيات الدائرية والشيفرة الميتة قبل الإيداع. تكوين .pre-commit-config.yaml جاهز للاستخدام.',
      pl: 'Uzyj AigisCode jako hooka pre-commit, aby wychwycic cykliczne zaleznosci i martwy kod przed commitem. Gotowa konfiguracja .pre-commit-hooks.yaml do ukierunkowanej, szybkiej analizy.',
      bn: 'কমিটের আগে সার্কুলার ডিপেন্ডেন্সি এবং ডেড কোড ধরতে AigisCode pre-commit hook হিসেবে ব্যবহার করুন। প্রস্তুত .pre-commit-config.yaml কনফিগারেশন।',
    },
    content: {
      en: `<h2 id="shift-left">Shift Left with Pre-commit</h2>
<p>The earlier you catch an architectural issue, the cheaper it is to fix. Pre-commit hooks run before the code even leaves the developer's machine, making them the fastest feedback loop available. AigisCode integrates with the popular <a href="https://pre-commit.com" target="_blank" rel="noopener noreferrer">pre-commit</a> framework to provide this early detection.</p>
<p>When a developer runs <code>git commit</code>, AigisCode analyzes the staged changes and blocks the commit if it introduces new circular dependencies, adds dead imports, or embeds hardwired values. The developer gets immediate feedback and can fix the issue before pushing.</p>

<h2 id="pre-commit-config">Configuration</h2>
<p>Add AigisCode to your <code>.pre-commit-config.yaml</code>:</p>
<pre><code>repos:
  - repo: https://github.com/Draivix/aigiscode
    rev: v0.1.0  # Use the latest release tag
    hooks:
      - id: aigiscode
        name: AigisCode Analysis
        entry: aigiscode analyze .
        language: python
        pass_filenames: false
        always_run: true
        stages: [pre-commit]</code></pre>
<p>Then install the hook:</p>
<pre><code># Install pre-commit if you haven't already
pip install pre-commit

# Install the hooks defined in your config
pre-commit install</code></pre>

<h2 id="how-it-works">How It Works</h2>
<p>When you run <code>git commit</code>, the pre-commit framework invokes AigisCode with the <code>--skip-ai</code> flag for fast, deterministic analysis. The hook:</p>
<ol>
<li>Analyzes your entire project (not just staged files, because architectural issues span multiple files).</li>
<li>Compares findings against your configured thresholds in <code>.aigiscode/policy.json</code>.</li>
<li>Returns exit code 0 (commit proceeds) or exit code 1 (commit blocked) based on whether new issues exceed thresholds.</li>
</ol>
<p>Because the analysis runs on the full project, it catches issues that file-level hooks would miss — like a new import that completes a circular dependency cycle across three files.</p>

<h2 id="performance">Performance Considerations</h2>
<p>Running a full project analysis on every commit may add 5-30 seconds depending on project size. To keep the feedback loop fast:</p>
<ul>
<li>Use <code>--skip-ai</code> to avoid API calls and run only deterministic detectors.</li>
<li>Configure <code>.aigiscode/policy.json</code> to exclude directories that change rarely (vendor, node_modules are excluded by default).</li>
<li>Use <code>stages: [pre-push]</code> instead of <code>stages: [pre-commit]</code> if the analysis takes too long for your workflow.</li>
</ul>
<p>For very large monorepos, consider running AigisCode as a pre-push hook or in CI instead, and use lighter pre-commit hooks (like ESLint or ruff) for file-level checks.</p>

<h2 id="skip-hook">Skipping the Hook</h2>
<p>In rare cases where you need to bypass the check (e.g., a work-in-progress commit on a feature branch), you can skip it with:</p>
<pre><code>git commit --no-verify -m "WIP: partial refactor"</code></pre>
<p>This is standard pre-commit behavior and should be used sparingly. The CI pipeline will still catch issues on push.</p>`,
      cs: `<h2 id="shift-left">Shift Left s Pre-commit</h2>
<p>Čím dříve zachytíte architektonický problém, tím levnější je jeho oprava. Pre-commit hooky běží před tím, než kód opustí vývojářův stroj. AigisCode se integruje s frameworkem <a href="https://pre-commit.com" target="_blank" rel="noopener noreferrer">pre-commit</a>.</p>

<h2 id="pre-commit-config">Konfigurace</h2>
<p>Přidejte AigisCode do <code>.pre-commit-config.yaml</code> s konfigurací hooku, který spustí <code>aigiscode analyze .</code>. Poté nainstalujte: <code>pre-commit install</code>.</p>

<h2 id="how-it-works">Jak to funguje</h2>
<p>Při <code>git commit</code> AigisCode analyzuje celý projekt (nejen staged soubory), porovná nálezy s prahy v <code>.aigiscode/policy.json</code> a vrátí exit kód 0 (commit pokračuje) nebo 1 (commit zablokován).</p>

<h2 id="performance">Výkon</h2>
<ul>
<li>Použijte <code>--skip-ai</code> pro vynechání API volání.</li>
<li>Konfigurujte vyloučení adresářů v <code>.aigiscode/policy.json</code>.</li>
<li>Pro velké monorepa zvažte <code>stages: [pre-push]</code> místo <code>stages: [pre-commit]</code>.</li>
</ul>

<h2 id="skip-hook">Přeskočení hooku</h2>
<p>V ojedinělých případech: <code>git commit --no-verify -m "WIP"</code>. CI pipeline problémy stále zachytí.</p>`,
      fr: `<h2 id="shift-left">Shift Left avec Pre-commit</h2>
<p>Plus tot vous detectez un probleme architectural, moins il coute a corriger. Les hooks pre-commit s'executent avant que le code ne quitte la machine du developpeur. AigisCode s'integre au framework <a href="https://pre-commit.com" target="_blank" rel="noopener noreferrer">pre-commit</a>.</p>

<h2 id="pre-commit-config">Configuration</h2>
<p>Ajoutez AigisCode a <code>.pre-commit-config.yaml</code> avec un hook qui execute <code>aigiscode analyze .</code>. Puis installez : <code>pre-commit install</code>.</p>

<h2 id="how-it-works">Comment ca fonctionne</h2>
<p>Lors du <code>git commit</code>, AigisCode analyse le projet entier (pas seulement les fichiers stages), compare les resultats aux seuils dans <code>.aigiscode/policy.json</code> et retourne le code de sortie 0 (commit accepte) ou 1 (commit bloque).</p>

<h2 id="performance">Performance</h2>
<ul>
<li>Utilisez <code>--skip-ai</code> pour eviter les appels API.</li>
<li>Configurez les exclusions de repertoires dans <code>.aigiscode/policy.json</code>.</li>
<li>Pour les grands monorepos, utilisez <code>stages: [pre-push]</code> au lieu de <code>stages: [pre-commit]</code>.</li>
</ul>

<h2 id="skip-hook">Ignorer le hook</h2>
<p>En cas rare : <code>git commit --no-verify -m "WIP"</code>. Le pipeline CI detectera les problemes.</p>`,
      es: `<h2 id="shift-left">Shift Left con Pre-commit</h2>
<p>Cuanto antes detecte un problema arquitectonico, mas barato es corregirlo. Los hooks pre-commit se ejecutan antes de que el codigo salga de la maquina del desarrollador. AigisCode se integra con el framework <a href="https://pre-commit.com" target="_blank" rel="noopener noreferrer">pre-commit</a>.</p>

<h2 id="pre-commit-config">Configuracion</h2>
<p>Agregue AigisCode a <code>.pre-commit-config.yaml</code> con un hook que ejecute <code>aigiscode analyze .</code>. Luego instale: <code>pre-commit install</code>.</p>

<h2 id="how-it-works">Como funciona</h2>
<p>Al hacer <code>git commit</code>, AigisCode analiza el proyecto completo (no solo archivos staged), compara hallazgos con umbrales en <code>.aigiscode/policy.json</code> y retorna codigo de salida 0 (commit procede) o 1 (commit bloqueado).</p>

<h2 id="performance">Rendimiento</h2>
<ul>
<li>Use <code>--skip-ai</code> para evitar llamadas API.</li>
<li>Configure exclusiones de directorios en <code>.aigiscode/policy.json</code>.</li>
<li>Para monorepos grandes, use <code>stages: [pre-push]</code> en lugar de <code>stages: [pre-commit]</code>.</li>
</ul>

<h2 id="skip-hook">Saltar el hook</h2>
<p>En casos raros: <code>git commit --no-verify -m "WIP"</code>. El pipeline CI detectara los problemas.</p>`,
      zh: `<h2 id="shift-left">使用 Pre-commit 左移</h2>
<p>越早发现架构问题，修复成本越低。Pre-commit 钩子在代码离开开发者机器之前运行。AigisCode 与 <a href="https://pre-commit.com" target="_blank" rel="noopener noreferrer">pre-commit</a> 框架集成。</p>

<h2 id="pre-commit-config">配置</h2>
<p>在 <code>.pre-commit-config.yaml</code> 中添加 AigisCode，配置运行 <code>aigiscode analyze .</code> 的钩子。然后安装：<code>pre-commit install</code>。</p>

<h2 id="how-it-works">工作原理</h2>
<p><code>git commit</code> 时，AigisCode 分析整个项目（不仅是暂存文件），将发现与 <code>.aigiscode/policy.json</code> 中的阈值比较，返回退出码 0（提交继续）或 1（提交被阻止）。</p>

<h2 id="performance">性能考虑</h2>
<ul>
<li>使用 <code>--skip-ai</code> 避免 API 调用。</li>
<li>在 <code>.aigiscode/policy.json</code> 中配置目录排除。</li>
<li>对于大型 monorepo，使用 <code>stages: [pre-push]</code> 代替 <code>stages: [pre-commit]</code>。</li>
</ul>

<h2 id="skip-hook">跳过钩子</h2>
<p>在极少数情况下：<code>git commit --no-verify -m "WIP"</code>。CI 流水线仍会捕获问题。</p>`,
      hi: `<h2 id="shift-left">Pre-commit के साथ शिफ्ट लेफ्ट</h2>
<p>जितनी जल्दी आप आर्किटेक्चरल समस्या पकड़ते हैं, इसे ठीक करना उतना सस्ता होता है। Pre-commit हुक कोड के डेवलपर की मशीन छोड़ने से पहले चलते हैं। AigisCode <a href="https://pre-commit.com" target="_blank" rel="noopener noreferrer">pre-commit</a> फ्रेमवर्क के साथ इंटीग्रेट होता है।</p>

<h2 id="pre-commit-config">कॉन्फ़िगरेशन</h2>
<p><code>.pre-commit-config.yaml</code> में AigisCode जोड़ें जो <code>aigiscode analyze .</code> चलाता है। फिर इंस्टॉल करें: <code>pre-commit install</code>।</p>

<h2 id="how-it-works">कैसे काम करता है</h2>
<p><code>git commit</code> पर, AigisCode पूरे प्रोजेक्ट का विश्लेषण करता है (केवल staged फ़ाइलें नहीं), <code>.aigiscode/policy.json</code> में थ्रेशोल्ड से तुलना करता है और exit कोड 0 (कमिट आगे बढ़ता है) या 1 (कमिट ब्लॉक) रिटर्न करता है।</p>

<h2 id="performance">परफ़ॉर्मेंस</h2>
<ul>
<li>API कॉल से बचने के लिए <code>--skip-ai</code> का उपयोग करें।</li>
<li><code>.aigiscode/policy.json</code> में डायरेक्टरी एक्सक्लूज़न कॉन्फ़िगर करें।</li>
<li>बड़े monorepos के लिए, <code>stages: [pre-commit]</code> के बजाय <code>stages: [pre-push]</code> का उपयोग करें।</li>
</ul>

<h2 id="skip-hook">हुक स्किप करना</h2>
<p>दुर्लभ मामलों में: <code>git commit --no-verify -m "WIP"</code>। CI पाइपलाइन अभी भी समस्याएं पकड़ेगी।</p>`,
      pt: `<h2 id="shift-left">Shift Left com Pre-commit</h2>
<p>Quanto mais cedo voce detectar um problema arquitetural, mais barato e corrigi-lo. Hooks pre-commit executam antes do codigo sair da maquina do desenvolvedor. O AigisCode integra-se com o framework <a href="https://pre-commit.com" target="_blank" rel="noopener noreferrer">pre-commit</a>.</p>

<h2 id="pre-commit-config">Configuracao</h2>
<p>Adicione o AigisCode ao <code>.pre-commit-config.yaml</code> com um hook que executa <code>aigiscode analyze .</code>. Depois instale: <code>pre-commit install</code>.</p>

<h2 id="how-it-works">Como funciona</h2>
<p>No <code>git commit</code>, o AigisCode analisa o projeto inteiro (nao apenas arquivos staged), compara descobertas com limites em <code>.aigiscode/policy.json</code> e retorna codigo de saida 0 (commit prossegue) ou 1 (commit bloqueado).</p>

<h2 id="performance">Performance</h2>
<ul>
<li>Use <code>--skip-ai</code> para evitar chamadas de API.</li>
<li>Configure exclusoes de diretorios em <code>.aigiscode/policy.json</code>.</li>
<li>Para grandes monorepos, use <code>stages: [pre-push]</code> em vez de <code>stages: [pre-commit]</code>.</li>
</ul>

<h2 id="skip-hook">Pular o hook</h2>
<p>Em casos raros: <code>git commit --no-verify -m "WIP"</code>. O pipeline CI ainda detectara os problemas.</p>`,
      ar: `<h2 id="shift-left">التحول لليسار مع pre-commit</h2>
<p>اكتشف المشكلات المعمارية في أقرب وقت ممكن — قبل الإيداع. يتكامل AigisCode مع إطار pre-commit لتحليل الملفات المرحلية ورفض الإيداعات التي تُدخل مشكلات جديدة.</p>

<h2 id="configuration">التكوين</h2>
<p>أضف AigisCode إلى <code>.pre-commit-config.yaml</code>. يعمل الخطاف على الملفات المرحلية فقط مما يبقيه سريعاً. علامة <code>--skip-ai</code> تضمن عدم وجود استدعاءات API أثناء الإيداعات.</p>

<h2 id="what-it-catches">ما يكتشفه</h2>
<ul>
<li><strong>التبعيات الدائرية</strong> — منع إضافة استيرادات تخلق دورات.</li>
<li><strong>الشيفرة الميتة</strong> — اكتشاف الاستيرادات غير المستخدمة والملفات اليتيمة.</li>
<li><strong>انتهاكات الطبقات</strong> — منع الاستيرادات عبر الطبقات.</li>
<li><strong>القيم الثابتة</strong> — اكتشاف عناوين URLs المشفّرة والسلاسل السحرية.</li>
</ul>

<h2 id="skipping">تخطي الخطاف</h2>
<p>عند الحاجة يمكن تخطي الخطاف بـ <code>git commit --no-verify</code>. يظل التحليل الكامل في CI كشبكة أمان.</p>`,
      pl: `<h2 id="pre-commit">Hooki pre-commit</h2>
<p>AigisCode integruje sie z frameworkiem pre-commit. Ukierunkowana analiza na plikach staged podczas commita.</p>`,
    },
    features: [
      'Pre-commit framework integration',
      'Block commits with issues',
      'Full project analysis',
      'Deterministic --skip-ai mode',
      'Configurable thresholds',
      'Pre-push hook alternative',
    ],
    codeExample: `# .pre-commit-config.yaml
repos:
  - repo: https://github.com/Draivix/aigiscode
    rev: v0.1.0
    hooks:
      - id: aigiscode
        name: AigisCode Analysis
        entry: aigiscode analyze .
        language: python
        pass_filenames: false
        always_run: true

# Install
pip install pre-commit
pre-commit install`,
    relatedSlugs: ['ci-cd-pipelines', 'github-actions', 'ide-plugins'],
  },

  /* ---- 7. REST API ---- */
  {
    slug: 'rest-api',
    icon: 'Plugs',
    category: 'Developer Tools',
    status: 'coming-soon',
    title: {
      en: 'REST API',
      cs: 'REST API',
      fr: 'API REST',
      es: 'API REST',
      zh: 'REST API',
      hi: 'REST API',
      pt: 'API REST',
      ar: 'واجهة REST API',
      pl: 'REST API',
      bn: 'REST API',
    },
    shortDescription: {
      en: 'Programmatic access to AigisCode analysis through a REST API. Submit repositories for analysis, retrieve reports, and integrate with custom dashboards and tools.',
      cs: 'Programový přístup k analýze AigisCode prostřednictvím REST API. Odesílejte repozitáře k analýze, načítejte reporty a integrujte s vlastními dashboardy a nástroji.',
      fr: 'Accès programmatique à l\'analyse AigisCode via une API REST. Soumettez des dépôts pour analyse, récupérez les rapports et intégrez-les à vos tableaux de bord et outils personnalisés.',
      es: 'Acceso programático al análisis de AigisCode a través de una API REST. Envíe repositorios para análisis, recupere informes e integre con dashboards y herramientas personalizadas.',
      zh: '通过 REST API 以编程方式访问 AigisCode 分析。提交仓库进行分析、检索报告并与自定义仪表板和工具集成。',
      hi: 'REST API के माध्यम से AigisCode विश्लेषण तक प्रोग्रामेटिक पहुंच। विश्लेषण के लिए रिपॉज़िटरी सबमिट करें, रिपोर्ट प्राप्त करें और कस्टम डैशबोर्ड और टूल्स के साथ इंटीग्रेट करें।',
      pt: 'Acesso programático à análise do AigisCode através de uma API REST. Envie repositórios para análise, recupere relatórios e integre com dashboards e ferramentas personalizadas.',
      ar: 'وصول برمجي لتحليل AigisCode عبر واجهة REST API. أرسل المستودعات للتحليل واسترجع التقارير وادمج مع لوحات المعلومات والأدوات المخصصة.',
      pl: 'Programowy dostep do analizy AigisCode przez REST API. Przesylaj repozytoria do analizy, pobieraj raporty i buduj wlasne integracje z ustrukturyzowanymi odpowiedziami JSON.',
      bn: 'REST API-এর মাধ্যমে AigisCode বিশ্লেষণে প্রোগ্রামেটিক অ্যাক্সেস। বিশ্লেষণের জন্য রিপোজিটরি জমা দিন, রিপোর্ট পুনরুদ্ধার করুন এবং কাস্টম ড্যাশবোর্ড ও টুলসের সাথে ইন্টিগ্রেট করুন।',
    },
    metaDescription: {
      en: 'AigisCode REST API for programmatic codebase analysis. Submit repos, retrieve reports, and build custom integrations. Coming soon.',
      cs: 'REST API AigisCode pro programovou analýzu kódové báze. Odesílejte repozitáře, načítejte reporty a vytvářejte vlastní integrace. Již brzy.',
      fr: 'API REST AigisCode pour l\'analyse programmatique de bases de code. Soumettez des dépôts, récupérez des rapports et créez des intégrations personnalisées. Bientôt disponible.',
      es: 'API REST de AigisCode para análisis programático de bases de código. Envíe repos, recupere informes y construya integraciones personalizadas. Próximamente.',
      zh: 'AigisCode REST API，用于编程方式的代码库分析。提交仓库、检索报告并构建自定义集成。即将推出。',
      hi: 'प्रोग्रामेटिक कोडबेस विश्लेषण के लिए AigisCode REST API। रिपॉज़ सबमिट करें, रिपोर्ट प्राप्त करें और कस्टम इंटीग्रेशन बनाएं। जल्द आ रहा है।',
      pt: 'API REST do AigisCode para análise programática de bases de código. Envie repos, recupere relatórios e construa integrações personalizadas. Em breve.',
      ar: 'واجهة REST API من AigisCode للتحليل البرمجي لقاعدة الشيفرة. أرسل المستودعات واسترجع التقارير وابنِ تكاملات مخصصة. قريباً.',
      pl: 'REST API AigisCode do programowej analizy bazy kodu. Przesylaj repozytoria, pobieraj raporty i buduj wlasne integracje z ustrukturyzowanymi odpowiedziami JSON.',
      bn: 'প্রোগ্রামেটিক কোডবেস বিশ্লেষণের জন্য AigisCode REST API। রিপো জমা দিন, রিপোর্ট পুনরুদ্ধার করুন এবং কাস্টম ইন্টিগ্রেশন তৈরি করুন। শীঘ্রই আসছে।',
    },
    content: {
      en: `<h2 id="api-overview">Programmatic Access to AigisCode</h2>
<p>The AigisCode REST API will provide a hosted analysis service that you can integrate into any workflow. Instead of installing AigisCode locally or in your CI environment, you will be able to submit a repository URL or archive and receive a structured JSON report through a simple HTTP interface.</p>
<p>This enables use cases that go beyond CI/CD — custom dashboards, portfolio-wide analysis tools, architectural health monitoring services, and integrations with project management platforms.</p>

<h2 id="planned-endpoints">Planned Endpoints</h2>
<p>The API will expose the following core endpoints:</p>
<ul>
<li><strong>POST /v1/analyze</strong> — Submit a repository URL or archive for analysis. Returns a job ID for polling.</li>
<li><strong>GET /v1/jobs/{id}</strong> — Check the status of an analysis job (queued, running, completed, failed).</li>
<li><strong>GET /v1/reports/{id}</strong> — Retrieve the full JSON report for a completed analysis.</li>
<li><strong>GET /v1/reports/{id}/summary</strong> — Retrieve just the summary metrics without the full finding details.</li>
<li><strong>POST /v1/reports/{id}/compare</strong> — Compare two reports to identify new, resolved, and persistent findings.</li>
</ul>

<h2 id="authentication">Authentication</h2>
<p>The API will use API key authentication passed via the <code>Authorization</code> header. Keys will be scoped to organizations and support fine-grained permissions for read-only access, analysis submission, and policy management.</p>

<h2 id="use-cases">Planned Use Cases</h2>
<p>The REST API will enable several advanced workflows:</p>
<ul>
<li><strong>Portfolio monitoring</strong> — Analyze all repositories in an organization on a schedule and track architectural health trends across the portfolio.</li>
<li><strong>Custom dashboards</strong> — Build internal dashboards that visualize AigisCode findings alongside other quality metrics.</li>
<li><strong>Chatbot integration</strong> — Connect to Slack, Teams, or Discord bots that can trigger analysis and report results in conversation.</li>
<li><strong>Code review automation</strong> — Trigger analysis on PR events via webhooks and post findings directly in your code review tool.</li>
</ul>

<h2 id="early-access">Early Access</h2>
<p>The REST API is currently in development. In the meantime, use the CLI (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>) for local and CI analysis. The CLI output format is identical to what the API will return, so any integration you build today will work with the API when it launches.</p>`,
      cs: `<h2 id="api-overview">Programový přístup k AigisCode</h2>
<p>REST API AigisCode poskytne hostovanou analytickou službu, kterou můžete integrovat do jakéhokoliv workflow. Místo lokální instalace odešlete URL repozitáře a obdržíte strukturovaný JSON report přes jednoduché HTTP rozhraní.</p>

<h2 id="planned-endpoints">Plánované endpointy</h2>
<ul>
<li><strong>POST /v1/analyze</strong> — Odeslání repozitáře k analýze. Vrací ID úlohy.</li>
<li><strong>GET /v1/jobs/{id}</strong> — Kontrola stavu úlohy.</li>
<li><strong>GET /v1/reports/{id}</strong> — Získání plného JSON reportu.</li>
<li><strong>GET /v1/reports/{id}/summary</strong> — Pouze souhrnné metriky.</li>
<li><strong>POST /v1/reports/{id}/compare</strong> — Porovnání dvou reportů.</li>
</ul>

<h2 id="authentication">Autentizace</h2>
<p>API bude používat autentizaci pomocí API klíčů přes hlavičku <code>Authorization</code> s oprávněními na úrovni organizace.</p>

<h2 id="early-access">Předběžný přístup</h2>
<p>REST API je ve vývoji. Mezitím použijte CLI (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>). Formát výstupu CLI je identický s tím, co vrátí API.</p>`,
      fr: `<h2 id="api-overview">Acces programmatique a AigisCode</h2>
<p>L'API REST AigisCode fournira un service d'analyse heberge integrable dans n'importe quel workflow. Au lieu d'installer localement, soumettez une URL de depot et recevez un rapport JSON structure via une interface HTTP simple.</p>

<h2 id="planned-endpoints">Endpoints prevus</h2>
<ul>
<li><strong>POST /v1/analyze</strong> — Soumettre un depot pour analyse. Retourne un ID de job.</li>
<li><strong>GET /v1/jobs/{id}</strong> — Verifier le statut du job.</li>
<li><strong>GET /v1/reports/{id}</strong> — Recuperer le rapport JSON complet.</li>
<li><strong>GET /v1/reports/{id}/summary</strong> — Seulement les metriques resumees.</li>
<li><strong>POST /v1/reports/{id}/compare</strong> — Comparer deux rapports.</li>
</ul>

<h2 id="authentication">Authentification</h2>
<p>L'API utilisera l'authentification par cle API via l'en-tete <code>Authorization</code> avec des permissions au niveau de l'organisation.</p>

<h2 id="early-access">Acces anticipe</h2>
<p>L'API REST est en developpement. En attendant, utilisez le CLI (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>). Le format de sortie du CLI est identique a celui de l'API.</p>`,
      es: `<h2 id="api-overview">Acceso programatico a AigisCode</h2>
<p>La API REST de AigisCode proporcionara un servicio de analisis hospedado que puede integrar en cualquier workflow. En lugar de instalar localmente, envie una URL de repositorio y reciba un reporte JSON estructurado mediante una interfaz HTTP simple.</p>

<h2 id="planned-endpoints">Endpoints planificados</h2>
<ul>
<li><strong>POST /v1/analyze</strong> — Enviar repositorio para analisis. Retorna un ID de job.</li>
<li><strong>GET /v1/jobs/{id}</strong> — Verificar estado del job.</li>
<li><strong>GET /v1/reports/{id}</strong> — Obtener el reporte JSON completo.</li>
<li><strong>GET /v1/reports/{id}/summary</strong> — Solo metricas resumidas.</li>
<li><strong>POST /v1/reports/{id}/compare</strong> — Comparar dos reportes.</li>
</ul>

<h2 id="authentication">Autenticacion</h2>
<p>La API usara autenticacion por clave API via encabezado <code>Authorization</code> con permisos a nivel de organizacion.</p>

<h2 id="early-access">Acceso anticipado</h2>
<p>La API REST esta en desarrollo. Mientras tanto, use el CLI (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>). El formato de salida del CLI es identico al de la API.</p>`,
      zh: `<h2 id="api-overview">对 AigisCode 的编程访问</h2>
<p>AigisCode REST API 将提供托管分析服务，可集成到任何工作流中。无需本地安装，提交仓库 URL 即可通过简单的 HTTP 接口接收结构化 JSON 报告。</p>

<h2 id="planned-endpoints">计划的端点</h2>
<ul>
<li><strong>POST /v1/analyze</strong> — 提交仓库进行分析。返回作业 ID。</li>
<li><strong>GET /v1/jobs/{id}</strong> — 检查作业状态。</li>
<li><strong>GET /v1/reports/{id}</strong> — 获取完整 JSON 报告。</li>
<li><strong>GET /v1/reports/{id}/summary</strong> — 仅摘要指标。</li>
<li><strong>POST /v1/reports/{id}/compare</strong> — 比较两份报告。</li>
</ul>

<h2 id="authentication">认证</h2>
<p>API 将使用通过 <code>Authorization</code> 头传递的 API 密钥认证，支持组织级权限。</p>

<h2 id="early-access">抢先体验</h2>
<p>REST API 正在开发中。同时请使用 CLI（<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>）。CLI 输出格式与 API 返回格式相同。</p>`,
      hi: `<h2 id="api-overview">AigisCode तक प्रोग्रामेटिक एक्सेस</h2>
<p>AigisCode REST API एक होस्टेड एनालिसिस सर्विस प्रदान करेगा जिसे आप किसी भी वर्कफ़्लो में इंटीग्रेट कर सकते हैं। लोकल इंस्टॉलेशन के बजाय, रिपॉज़िटरी URL सबमिट करें और सरल HTTP इंटरफ़ेस के माध्यम से स्ट्रक्चर्ड JSON रिपोर्ट प्राप्त करें।</p>

<h2 id="planned-endpoints">प्लान्ड एंडपॉइंट</h2>
<ul>
<li><strong>POST /v1/analyze</strong> — एनालिसिस के लिए रिपॉज़िटरी सबमिट करें। जॉब ID रिटर्न करता है।</li>
<li><strong>GET /v1/jobs/{id}</strong> — जॉब स्टेटस चेक करें।</li>
<li><strong>GET /v1/reports/{id}</strong> — पूरा JSON रिपोर्ट प्राप्त करें।</li>
<li><strong>GET /v1/reports/{id}/summary</strong> — केवल सारांश मेट्रिक्स।</li>
<li><strong>POST /v1/reports/{id}/compare</strong> — दो रिपोर्ट की तुलना करें।</li>
</ul>

<h2 id="authentication">ऑथेंटिकेशन</h2>
<p>API <code>Authorization</code> हेडर के माध्यम से API कुंजी ऑथेंटिकेशन का उपयोग करेगा, ऑर्गनाइज़ेशन-लेवल परमिशन के साथ।</p>

<h2 id="early-access">अर्ली एक्सेस</h2>
<p>REST API विकास में है। तब तक, CLI का उपयोग करें (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>)। CLI आउटपुट फ़ॉर्मेट API रिटर्न फ़ॉर्मेट के समान है।</p>`,
      pt: `<h2 id="api-overview">Acesso programatico ao AigisCode</h2>
<p>A API REST do AigisCode fornecera um servico de analise hospedado que voce pode integrar em qualquer workflow. Em vez de instalar localmente, envie uma URL de repositorio e receba um relatorio JSON estruturado atraves de uma interface HTTP simples.</p>

<h2 id="planned-endpoints">Endpoints planejados</h2>
<ul>
<li><strong>POST /v1/analyze</strong> — Enviar repositorio para analise. Retorna ID do job.</li>
<li><strong>GET /v1/jobs/{id}</strong> — Verificar status do job.</li>
<li><strong>GET /v1/reports/{id}</strong> — Obter relatorio JSON completo.</li>
<li><strong>GET /v1/reports/{id}/summary</strong> — Apenas metricas resumidas.</li>
<li><strong>POST /v1/reports/{id}/compare</strong> — Comparar dois relatorios.</li>
</ul>

<h2 id="authentication">Autenticacao</h2>
<p>A API usara autenticacao por chave API via cabecalho <code>Authorization</code> com permissoes a nivel de organizacao.</p>

<h2 id="early-access">Acesso antecipado</h2>
<p>A API REST esta em desenvolvimento. Enquanto isso, use o CLI (<code>cargo install --git https://github.com/Draivix/aigiscode.git aigiscore --bin aigiscode</code>). O formato de saida do CLI e identico ao que a API retornara.</p>`,
      ar: `<h2 id="programmatic-access">الوصول البرمجي إلى AigisCode</h2>
<p>واجهة REST API تتيح الوصول البرمجي لتحليل AigisCode. أرسل المستودعات للتحليل واسترجع التقارير وادمج مع لوحات المعلومات والأدوات المخصصة.</p>

<h2 id="planned-endpoints">نقاط النهاية المخططة</h2>
<ul>
<li><strong>POST /analyze</strong> — إرسال مستودع للتحليل. يدعم عناوين URL لـ Git والملفات المرفوعة.</li>
<li><strong>GET /reports/{id}</strong> — استرجاع تقرير التحليل بتنسيق JSON.</li>
<li><strong>GET /reports/{id}/summary</strong> — استرجاع ملخص مكثف مع المقاييس الرئيسية.</li>
<li><strong>POST /compare</strong> — مقارنة تقريرين لاكتشاف الانحدارات.</li>
</ul>

<h2 id="authentication">المصادقة</h2>
<p>رموز API مع أذونات قابلة للتكوين. يتكامل مع OAuth لتطبيقات الفريق.</p>

<h2 id="early-access">الوصول المبكر</h2>
<p>واجهة REST API قيد التطوير. سجّل على <code>aigiscode.com</code> للوصول المبكر.</p>`,
      pl: `<h2 id="rest-api">REST API</h2>
<p>Programowy dostep do wynikow analizy. Przesylaj repozytoria, pobieraj raporty JSON i buduj wlasne integracje.</p>`,
    },
    features: [
      'Repository analysis endpoint',
      'Job status polling',
      'Report comparison API',
      'API key authentication',
      'Organization scoping',
      'Rate limiting and quotas',
    ],
    relatedSlugs: ['webhooks', 'ai-coding-agents', 'ci-cd-pipelines'],
  },

  /* ---- 8. Webhooks ---- */
  {
    slug: 'webhooks',
    icon: 'MagicWand',
    category: 'Developer Tools',
    status: 'coming-soon',
    title: {
      en: 'Webhooks',
      cs: 'Webhooky',
      fr: 'Webhooks',
      es: 'Webhooks',
      zh: 'Webhooks',
      hi: 'Webhooks',
      pt: 'Webhooks',
      ar: 'خطافات الويب',
      pl: 'Webhooki',
      bn: 'Webhook',
    },
    shortDescription: {
      en: 'Real-time notifications when AigisCode analysis completes. Receive structured payloads via HTTP POST to trigger downstream workflows and alerts.',
      cs: 'Oznámení v reálném čase po dokončení analýzy AigisCode. Přijímejte strukturované payloady přes HTTP POST pro spouštění navazujících workflow a upozornění.',
      fr: 'Notifications en temps réel à la fin de l\'analyse AigisCode. Recevez des payloads structurés via HTTP POST pour déclencher des workflows et des alertes en aval.',
      es: 'Notificaciones en tiempo real cuando el análisis de AigisCode se completa. Reciba payloads estructurados vía HTTP POST para activar workflows y alertas posteriores.',
      zh: 'AigisCode 分析完成时的实时通知。通过 HTTP POST 接收结构化数据包，以触发下游工作流和告警。',
      hi: 'AigisCode विश्लेषण पूरा होने पर रियल-टाइम नोटिफ़िकेशन। डाउनस्ट्रीम वर्कफ़्लो और अलर्ट ट्रिगर करने के लिए HTTP POST के माध्यम से स्ट्रक्चर्ड पेलोड प्राप्त करें।',
      pt: 'Notificações em tempo real quando a análise do AigisCode é concluída. Receba payloads estruturados via HTTP POST para acionar workflows e alertas subsequentes.',
      ar: 'إشعارات فورية عند اكتمال تحليل AigisCode. استقبل حمولات منظمة عبر HTTP POST لتشغيل سير العمل والتنبيهات اللاحقة.',
      pl: 'Powiadomienia w czasie rzeczywistym po zakonczeniu analizy AigisCode. Otrzymuj ustrukturyzowane dane przez HTTP POST na swoj endpoint, z podsumowaniami, krytycznymi znaleziskami i danymi trendow do dashboardow i alertow.',
      bn: 'AigisCode বিশ্লেষণ সম্পন্ন হলে রিয়েল-টাইম নোটিফিকেশন। ডাউনস্ট্রিম ওয়ার্কফ্লো এবং সতর্কতা ট্রিগার করতে HTTP POST-এর মাধ্যমে কাঠামোবদ্ধ পেলোড গ্রহণ করুন।',
    },
    metaDescription: {
      en: 'AigisCode webhooks for real-time analysis notifications. HTTP POST payloads with findings, configurable event filters, and retry logic. Coming soon.',
      cs: 'Webhooky AigisCode pro oznámení o analýze v reálném čase. HTTP POST payloady s nálezy, konfigurovatelné filtry událostí a logika opakování. Již brzy.',
      fr: 'Webhooks AigisCode pour les notifications d\'analyse en temps réel. Payloads HTTP POST avec résultats, filtres d\'événements configurables et logique de réessai. Bientôt disponible.',
      es: 'Webhooks de AigisCode para notificaciones de análisis en tiempo real. Payloads HTTP POST con hallazgos, filtros de eventos configurables y lógica de reintentos. Próximamente.',
      zh: 'AigisCode Webhooks，用于实时分析通知。包含发现结果的 HTTP POST 数据包、可配置的事件过滤器和重试逻辑。即将推出。',
      hi: 'रियल-टाइम विश्लेषण नोटिफ़िकेशन के लिए AigisCode Webhooks। खोज परिणामों के साथ HTTP POST पेलोड, कॉन्फ़िगर करने योग्य इवेंट फ़िल्टर और रिट्राई लॉजिक। जल्द आ रहा है।',
      pt: 'Webhooks do AigisCode para notificações de análise em tempo real. Payloads HTTP POST com achados, filtros de eventos configuráveis e lógica de retry. Em breve.',
      ar: 'خطافات الويب في AigisCode لإشعارات التحليل الفورية. حمولات HTTP POST مع النتائج وفلاتر أحداث قابلة للتكوين ومنطق إعادة المحاولة. قريباً.',
      pl: 'Webhooki AigisCode do powiadomien o analizie w czasie rzeczywistym. Dane HTTP POST ze znaleziskami, konfigurowalnymi progami i integracja ze Slack, Teams i wlasnymi endpointami.',
      bn: 'রিয়েল-টাইম বিশ্লেষণ নোটিফিকেশনের জন্য AigisCode webhook। ফলাফল সহ HTTP POST পেলোড, কনফিগারযোগ্য ইভেন্ট ফিল্টার এবং রিট্রাই লজিক। শীঘ্রই আসছে।',
    },
    content: {
      en: `<h2 id="webhook-overview">Real-Time Analysis Notifications</h2>
<p>Webhooks enable event-driven architectures where AigisCode pushes results to your systems instead of requiring you to poll for them. When an analysis completes, AigisCode will send an HTTP POST request to your configured URL with a structured JSON payload containing the report summary and findings.</p>
<p>This is the foundation for building reactive workflows — trigger a Slack notification when new circular dependencies are found, update a dashboard when the dead code count decreases, or create a Jira ticket when a god class exceeds a size threshold.</p>

<h2 id="payload-structure">Payload Structure</h2>
<p>Webhook payloads will follow a consistent envelope structure:</p>
<pre><code>{
  "event": "analysis.completed",
  "timestamp": "2026-03-09T14:30:00Z",
  "repository": "github.com/org/repo",
  "report_id": "rpt_abc123",
  "summary": {
    "total_files": 342,
    "circular_dependencies": 3,
    "dead_code": 47,
    "hardwired_values": 12,
    "status": "findings_detected"
  },
  "report_url": "https://api.aigiscode.com/v1/reports/rpt_abc123",
  "diff": {
    "new_findings": 2,
    "resolved_findings": 5,
    "persistent_findings": 55
  }
}</code></pre>
<p>The payload includes a summary for quick processing and a <code>report_url</code> to fetch the full report when detailed findings are needed.</p>

<h2 id="event-types">Planned Event Types</h2>
<p>The webhook system will support multiple event types that you can subscribe to individually:</p>
<ul>
<li><strong>analysis.completed</strong> — Fires when any analysis finishes, regardless of outcome.</li>
<li><strong>analysis.failed</strong> — Fires when analysis encounters an error (parse failure, timeout, configuration error).</li>
<li><strong>findings.new</strong> — Fires only when new findings are detected compared to the previous run.</li>
<li><strong>findings.resolved</strong> — Fires when previously detected findings are no longer present.</li>
<li><strong>threshold.exceeded</strong> — Fires when findings exceed your configured policy thresholds.</li>
</ul>

<h2 id="delivery-guarantees">Delivery and Retry Logic</h2>
<p>Webhooks will include built-in reliability features:</p>
<ul>
<li><strong>Automatic retries</strong> — Failed deliveries (5xx responses, timeouts) are retried up to 5 times with exponential backoff.</li>
<li><strong>Signature verification</strong> — Each payload includes an HMAC-SHA256 signature in the <code>X-AigisCode-Signature</code> header so you can verify the request is authentic.</li>
<li><strong>Delivery log</strong> — View recent webhook deliveries, response codes, and retry history in the AigisCode dashboard.</li>
<li><strong>Idempotency keys</strong> — Each delivery includes a unique <code>X-AigisCode-Delivery</code> header to prevent duplicate processing.</li>
</ul>

<h2 id="early-access">Early Access</h2>
<p>Webhooks are currently in development as part of the AigisCode platform. In the meantime, you can build similar notification workflows using CI/CD pipeline steps that parse the JSON report and send notifications via curl or a messaging SDK.</p>`,
      cs: `<h2 id="webhook-overview">Oznámení o analýze v reálném čase</h2>
<p>Webhooky umožňují event-driven architektury, kde AigisCode posílá výsledky do vašich systémů místo toho, abyste se na ně dotazovali. Po dokončení analýzy AigisCode odešle HTTP POST požadavek s JSON payloadem obsahujícím souhrn reportu a nálezy.</p>

<h2 id="payload-structure">Struktura payloadu</h2>
<p>Payload webhooků obsahuje: <code>event</code> (typ události), <code>timestamp</code>, <code>repository</code>, <code>report_id</code>, <code>summary</code> s metrikami a <code>diff</code> s novými, vyřešenými a přetrvávajícími nálezy. Obsahuje také <code>report_url</code> pro stažení plného reportu.</p>

<h2 id="event-types">Typy událostí</h2>
<ul>
<li><strong>analysis.completed</strong> — Spuštěno po dokončení jakékoliv analýzy.</li>
<li><strong>analysis.failed</strong> — Spuštěno při chybě analýzy.</li>
<li><strong>findings.new</strong> — Spuštěno pouze při detekci nových nálezů.</li>
<li><strong>findings.resolved</strong> — Spuštěno při vyřešení dříve detekovaných nálezů.</li>
<li><strong>threshold.exceeded</strong> — Spuštěno při překročení konfigurovaných prahů.</li>
</ul>

<h2 id="delivery-guarantees">Spolehlivost doručení</h2>
<p>Automatické opakování (až 5x s exponenciálním backoff), HMAC-SHA256 podpis v hlavičce <code>X-AigisCode-Signature</code>, log doručení a klíče idempotence.</p>

<h2 id="early-access">Předběžný přístup</h2>
<p>Webhooky jsou ve vývoji. Mezitím můžete podobné workflow vytvořit pomocí CI/CD kroků parsujících JSON report.</p>`,
      fr: `<h2 id="webhook-overview">Notifications d'analyse en temps reel</h2>
<p>Les webhooks permettent des architectures evenementielles ou AigisCode envoie les resultats a vos systemes. A la fin de l'analyse, AigisCode envoie une requete HTTP POST avec un payload JSON contenant le resume du rapport et les resultats.</p>

<h2 id="payload-structure">Structure du payload</h2>
<p>Le payload contient : <code>event</code> (type d'evenement), <code>timestamp</code>, <code>repository</code>, <code>report_id</code>, <code>summary</code> avec metriques et <code>diff</code> avec nouveaux resultats, resolus et persistants. Inclut aussi <code>report_url</code> pour le rapport complet.</p>

<h2 id="event-types">Types d'evenements</h2>
<ul>
<li><strong>analysis.completed</strong> — Declenche a la fin de toute analyse.</li>
<li><strong>analysis.failed</strong> — Declenche en cas d'erreur d'analyse.</li>
<li><strong>findings.new</strong> — Declenche uniquement lors de la detection de nouveaux resultats.</li>
<li><strong>findings.resolved</strong> — Declenche quand des resultats precedents sont resolus.</li>
<li><strong>threshold.exceeded</strong> — Declenche quand les seuils configures sont depasses.</li>
</ul>

<h2 id="delivery-guarantees">Fiabilite de livraison</h2>
<p>Tentatives automatiques (jusqu'a 5 avec backoff exponentiel), signature HMAC-SHA256 dans l'en-tete <code>X-AigisCode-Signature</code>, journal de livraison et cles d'idempotence.</p>

<h2 id="early-access">Acces anticipe</h2>
<p>Les webhooks sont en developpement. En attendant, creez des workflows similaires avec des etapes CI/CD parsant le rapport JSON.</p>`,
      es: `<h2 id="webhook-overview">Notificaciones de analisis en tiempo real</h2>
<p>Los webhooks permiten arquitecturas basadas en eventos donde AigisCode envia resultados a sus sistemas. Al completar un analisis, AigisCode envia una solicitud HTTP POST con un payload JSON que contiene el resumen del reporte y hallazgos.</p>

<h2 id="payload-structure">Estructura del payload</h2>
<p>El payload contiene: <code>event</code> (tipo de evento), <code>timestamp</code>, <code>repository</code>, <code>report_id</code>, <code>summary</code> con metricas y <code>diff</code> con hallazgos nuevos, resueltos y persistentes. Incluye <code>report_url</code> para el reporte completo.</p>

<h2 id="event-types">Tipos de eventos</h2>
<ul>
<li><strong>analysis.completed</strong> — Se activa al completar cualquier analisis.</li>
<li><strong>analysis.failed</strong> — Se activa en caso de error de analisis.</li>
<li><strong>findings.new</strong> — Se activa solo al detectar nuevos hallazgos.</li>
<li><strong>findings.resolved</strong> — Se activa cuando hallazgos previos se resuelven.</li>
<li><strong>threshold.exceeded</strong> — Se activa cuando se exceden umbrales configurados.</li>
</ul>

<h2 id="delivery-guarantees">Fiabilidad de entrega</h2>
<p>Reintentos automaticos (hasta 5 con backoff exponencial), firma HMAC-SHA256 en encabezado <code>X-AigisCode-Signature</code>, registro de entregas y claves de idempotencia.</p>

<h2 id="early-access">Acceso anticipado</h2>
<p>Los webhooks estan en desarrollo. Mientras tanto, cree workflows similares con pasos de CI/CD que parseen el reporte JSON.</p>`,
      zh: `<h2 id="webhook-overview">实时分析通知</h2>
<p>Webhooks 实现事件驱动架构，AigisCode 将结果推送到您的系统。分析完成后，AigisCode 发送包含报告摘要和发现结果的 JSON 数据包的 HTTP POST 请求。</p>

<h2 id="payload-structure">数据包结构</h2>
<p>数据包包含：<code>event</code>（事件类型）、<code>timestamp</code>、<code>repository</code>、<code>report_id</code>、带指标的 <code>summary</code> 和带新发现、已解决和持续发现的 <code>diff</code>。还包含获取完整报告的 <code>report_url</code>。</p>

<h2 id="event-types">事件类型</h2>
<ul>
<li><strong>analysis.completed</strong> — 任何分析完成时触发。</li>
<li><strong>analysis.failed</strong> — 分析出错时触发。</li>
<li><strong>findings.new</strong> — 仅在检测到新发现时触发。</li>
<li><strong>findings.resolved</strong> — 之前的发现被解决时触发。</li>
<li><strong>threshold.exceeded</strong> — 超出配置阈值时触发。</li>
</ul>

<h2 id="delivery-guarantees">投递可靠性</h2>
<p>自动重试（最多 5 次指数退避）、<code>X-AigisCode-Signature</code> 头中的 HMAC-SHA256 签名、投递日志和幂等键。</p>

<h2 id="early-access">抢先体验</h2>
<p>Webhooks 正在开发中。同时，您可以使用解析 JSON 报告的 CI/CD 步骤构建类似工作流。</p>`,
      hi: `<h2 id="webhook-overview">रियल-टाइम एनालिसिस नोटिफ़िकेशन</h2>
<p>Webhooks इवेंट-ड्रिवन आर्किटेक्चर सक्षम करते हैं जहां AigisCode आपके सिस्टम को रिजल्ट पुश करता है। एनालिसिस पूरा होने पर, AigisCode रिपोर्ट सारांश और फाइंडिंग के साथ JSON पेलोड का HTTP POST रिक्वेस्ट भेजता है।</p>

<h2 id="payload-structure">पेलोड स्ट्रक्चर</h2>
<p>पेलोड में शामिल हैं: <code>event</code> (इवेंट टाइप), <code>timestamp</code>, <code>repository</code>, <code>report_id</code>, मेट्रिक्स के साथ <code>summary</code> और नई, हल की गई और जारी फाइंडिंग के साथ <code>diff</code>। पूरी रिपोर्ट के लिए <code>report_url</code> भी शामिल है।</p>

<h2 id="event-types">इवेंट टाइप</h2>
<ul>
<li><strong>analysis.completed</strong> — किसी भी एनालिसिस पूरा होने पर ट्रिगर।</li>
<li><strong>analysis.failed</strong> — एनालिसिस में एरर होने पर ट्रिगर।</li>
<li><strong>findings.new</strong> — केवल नई फाइंडिंग डिटेक्ट होने पर ट्रिगर।</li>
<li><strong>findings.resolved</strong> — पहले की फाइंडिंग हल होने पर ट्रिगर।</li>
<li><strong>threshold.exceeded</strong> — कॉन्फ़िगर्ड थ्रेशोल्ड पार होने पर ट्रिगर।</li>
</ul>

<h2 id="delivery-guarantees">डिलीवरी विश्वसनीयता</h2>
<p>ऑटोमैटिक रिट्राई (एक्सपोनेंशियल बैकऑफ़ के साथ 5 बार तक), <code>X-AigisCode-Signature</code> हेडर में HMAC-SHA256 सिग्नेचर, डिलीवरी लॉग और इडेम्पोटेंसी कीज़।</p>

<h2 id="early-access">अर्ली एक्सेस</h2>
<p>Webhooks विकास में हैं। तब तक, JSON रिपोर्ट पार्स करने वाले CI/CD स्टेप्स के साथ समान वर्कफ़्लो बनाएं।</p>`,
      pt: `<h2 id="webhook-overview">Notificacoes de analise em tempo real</h2>
<p>Webhooks permitem arquiteturas orientadas a eventos onde o AigisCode envia resultados aos seus sistemas. Ao concluir uma analise, o AigisCode envia uma requisicao HTTP POST com um payload JSON contendo o resumo do relatorio e descobertas.</p>

<h2 id="payload-structure">Estrutura do payload</h2>
<p>O payload contem: <code>event</code> (tipo de evento), <code>timestamp</code>, <code>repository</code>, <code>report_id</code>, <code>summary</code> com metricas e <code>diff</code> com descobertas novas, resolvidas e persistentes. Inclui <code>report_url</code> para o relatorio completo.</p>

<h2 id="event-types">Tipos de evento</h2>
<ul>
<li><strong>analysis.completed</strong> — Disparado ao concluir qualquer analise.</li>
<li><strong>analysis.failed</strong> — Disparado em caso de erro na analise.</li>
<li><strong>findings.new</strong> — Disparado apenas ao detectar novas descobertas.</li>
<li><strong>findings.resolved</strong> — Disparado quando descobertas anteriores sao resolvidas.</li>
<li><strong>threshold.exceeded</strong> — Disparado quando limites configurados sao excedidos.</li>
</ul>

<h2 id="delivery-guarantees">Confiabilidade de entrega</h2>
<p>Tentativas automaticas (ate 5 com backoff exponencial), assinatura HMAC-SHA256 no cabecalho <code>X-AigisCode-Signature</code>, log de entregas e chaves de idempotencia.</p>

<h2 id="early-access">Acesso antecipado</h2>
<p>Webhooks estao em desenvolvimento. Enquanto isso, construa workflows similares com etapas de CI/CD que analisam o relatorio JSON.</p>`,
      ar: `<h2 id="real-time-notifications">إشعارات التحليل الفورية</h2>
<p>خطافات الويب في AigisCode ترسل إشعارات فورية عند اكتمال التحليل. استقبل حمولات منظمة عبر HTTP POST لتشغيل سير العمل والتنبيهات اللاحقة.</p>

<h2 id="planned-events">أنواع الأحداث المخططة</h2>
<ul>
<li><strong>analysis.completed</strong> — يُطلق عند اكتمال التحليل بنجاح. يتضمن عدد النتائج ومقاييس الشدة.</li>
<li><strong>analysis.failed</strong> — يُطلق عند فشل التحليل. يتضمن رسالة الخطأ والسياق.</li>
<li><strong>findings.new</strong> — يُطلق عند اكتشاف نتائج جديدة مقارنة بخط الأساس.</li>
<li><strong>findings.resolved</strong> — يُطلق عند حل نتائج سابقة.</li>
</ul>

<h2 id="payload-structure">هيكل الحمولة</h2>
<p>الحمولات بتنسيق JSON مع بيانات وصفية للحدث وتفاصيل التحليل وملخص النتائج. تُضمّن توقيعات HMAC-SHA256 للتحقق.</p>

<h2 id="delivery">التسليم وإعادة المحاولة</h2>
<p>HTTP POST مع إعادة محاولة أسّية (حتى ٣ محاولات). المهلة ٣٠ ثانية لكل تسليم. فلاتر الأحداث قابلة للتكوين لكل عنوان URL مسجل.</p>

<h2 id="use-cases">حالات الاستخدام المخططة</h2>
<ul>
<li><strong>إشعارات Slack/Teams</strong> — أرسل ملخصات التحليل إلى قنوات الدردشة.</li>
<li><strong>تحديث لوحة المعلومات</strong> — ادفع بالمقاييس إلى لوحات مراقبة مخصصة.</li>
<li><strong>إنشاء تذاكر</strong> — أنشئ تذاكر تلقائياً للنتائج عالية الشدة.</li>
<li><strong>سير عمل مخصص</strong> — شغّل خطوط أنابيب مخصصة بناءً على نتائج التحليل.</li>
</ul>

<h2 id="coming-soon">قريباً</h2>
<p>خطافات الويب قيد التطوير. تابع المستودع للتحديثات.</p>`,
      pl: `<h2 id="webhooks">Webhooki</h2>
<p>Powiadomienia w czasie rzeczywistym po zakonczeniu analizy. Integracja ze Slack, Teams lub wlasnymi endpointami.</p>`,
    },
    features: [
      'HTTP POST notifications',
      'Configurable event filters',
      'HMAC signature verification',
      'Automatic retry with backoff',
      'Diff summaries in payloads',
      'Delivery logging',
    ],
    relatedSlugs: ['rest-api', 'ai-coding-agents', 'github-actions'],
  },
];

/* -------------------------------------------------------------------------- */
/*  Helpers                                                                   */
/* -------------------------------------------------------------------------- */

export const integrationCategories = [...new Set(integrations.map((i) => i.category))];

export function getIntegrationBySlug(slug: string): Integration | undefined {
  return integrations.find((i) => i.slug === slug);
}

export function getRelatedIntegrations(slugs: string[]): Integration[] {
  return slugs
    .map((s) => integrations.find((i) => i.slug === s))
    .filter(Boolean) as Integration[];
}
