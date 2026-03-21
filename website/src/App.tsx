import { lazy, Suspense } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import ErrorBoundary from './components/ErrorBoundary';
import LanguageLayout from './components/LanguageLayout';
import Layout from './components/Layout';

const HomePage = lazy(() => import('./pages/HomePage'));
const BlogPage = lazy(() => import('./pages/BlogPage'));
const BlogPostPage = lazy(() => import('./pages/BlogPostPage'));
const UseCasesPage = lazy(() => import('./pages/UseCasesPage'));
const UseCasePage = lazy(() => import('./pages/UseCasePage'));
const DocsPage = lazy(() => import('./pages/DocsPage'));
const FeaturesPage = lazy(() => import('./pages/FeaturesPage'));
const FeaturePage = lazy(() => import('./pages/FeaturePage'));
const LanguagesPage = lazy(() => import('./pages/LanguagesPage'));
const LanguagePage = lazy(() => import('./pages/LanguagePage'));
const IntegrationsPage = lazy(() => import('./pages/IntegrationsPage'));
const IntegrationPage = lazy(() => import('./pages/IntegrationPage'));
const PlatformPage = lazy(() => import('./pages/PlatformPage'));
const PlatformFeaturePage = lazy(() => import('./pages/PlatformFeaturePage'));
const AboutPage = lazy(() => import('./pages/AboutPage'));
const CockpitDemoPage = lazy(() => import('./pages/CockpitDemoPage'));
const NotFoundPage = lazy(() => import('./pages/NotFoundPage'));

const PageLoader = (
  <div className="min-h-screen flex items-center justify-center">
    <div className="w-8 h-8 border-2 border-indigo-500/30 border-t-indigo-500 rounded-full animate-spin" />
  </div>
);

function Lazy({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={PageLoader}>{children}</Suspense>;
}

/* Page routes shared by both the default (no prefix) and /:lang prefix */
function PageRoutes() {
  return (
    <>
      <Route index element={<Lazy><HomePage /></Lazy>} />
      <Route path="docs" element={<Lazy><DocsPage /></Lazy>} />
      <Route path="blog" element={<Lazy><BlogPage /></Lazy>} />
      <Route path="blog/:slug" element={<Lazy><BlogPostPage /></Lazy>} />
      <Route path="use-cases" element={<Lazy><UseCasesPage /></Lazy>} />
      <Route path="use-cases/:slug" element={<Lazy><UseCasePage /></Lazy>} />
      <Route path="features" element={<Lazy><FeaturesPage /></Lazy>} />
      <Route path="features/:slug" element={<Lazy><FeaturePage /></Lazy>} />
      <Route path="languages" element={<Lazy><LanguagesPage /></Lazy>} />
      <Route path="languages/:slug" element={<Lazy><LanguagePage /></Lazy>} />
      <Route path="integrations" element={<Lazy><IntegrationsPage /></Lazy>} />
      <Route path="integrations/:slug" element={<Lazy><IntegrationPage /></Lazy>} />
      <Route path="platform" element={<Lazy><PlatformPage /></Lazy>} />
      <Route path="platform/:slug" element={<Lazy><PlatformFeaturePage /></Lazy>} />
      <Route path="about" element={<Lazy><AboutPage /></Lazy>} />
      <Route path="cockpit-demo" element={<Lazy><CockpitDemoPage /></Lazy>} />
      <Route path="*" element={<Lazy><NotFoundPage /></Lazy>} />
    </>
  );
}

export default function App() {
  return (
    <ErrorBoundary>
    <BrowserRouter>
      <Routes>
        {/* English (default) — no language prefix */}
        <Route element={<LanguageLayout />}>
          <Route element={<Layout />}>
            {PageRoutes()}
          </Route>
        </Route>

        {/* Non-English — /:lang prefix (cs, fr, es, zh, hi, pt, pl, ar, bn) */}
        <Route path=":lang" element={<LanguageLayout />}>
          <Route element={<Layout />}>
            {PageRoutes()}
          </Route>
        </Route>
      </Routes>
    </BrowserRouter>
    </ErrorBoundary>
  );
}
