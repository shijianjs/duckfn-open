import {createElement} from 'react';
import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Translate, {translate} from '@docusaurus/Translate';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import CodeBlock from '@theme/CodeBlock';
import Heading from '@theme/Heading';
import Layout from '@theme/Layout';
import {
  DfkFeatures,
  DfkHero,
  DfkNextSteps,
  registerDfkElements,
  type FeatureItem,
  type HeroAction,
  type HeroBadge,
  type HeroLink,
  type NextStepItem,
} from 'duckfn-docs-kit';

import styles from './index.module.css';

// Defining the `dfk-*` custom elements is a one-time side effect (it also
// registers the official `<iconify-icon>` element). Idempotent, and a no-op
// during Docusaurus' Node prerender pass.
registerDfkElements();

/**
 * The landing page, written for people who want to *use* the extension: hero,
 * features, an install/usage SQL showcase and the "where next" cards.
 *
 * The hero, the feature grid and the next-step cards are `dfk-*` web components
 * from duckfn-docs-kit, so the whole landing layout is reusable by other
 * extension docs sites. A custom element cannot render React's `<Translate>`,
 * so the copy is resolved with the imperative `translate()` API into plain
 * strings for the active locale and handed to the components through their own
 * setters; the strings still live in `i18n/zh-Hans/code.json` under the same
 * `homepage.*` keys. The code showcase stays here because it needs the theme's
 * `CodeBlock`.
 *
 * Known trade-off (accepted): the `dfk-*` sections render client-side, so their
 * prerendered HTML is empty until hydration — the same behaviour as the
 * `<iconify-icon>` glyphs.
 */

type DfkTag = 'dfk-hero' | 'dfk-features' | 'dfk-next-steps';

/**
 * Mounts a `dfk-*` element and hands its content over through the component's
 * own setters.
 *
 * React's SSR/hydration path only reconciles string/number props onto custom
 * elements — an object payload is never serialised into the prerendered HTML,
 * so hydration would leave the component empty. A callback ref is the reliable
 * channel: React calls it with the live node after mount, where the setters are
 * invoked. The components are retained-mode, so each setter only mutates the
 * nodes it owns; nothing is re-rendered.
 */
function dfk<TContent>(
  tag: DfkTag,
  mount: (node: HTMLElement, content: TContent) => void,
  content: TContent,
): ReactNode {
  return createElement(tag, {
    ref: (node: HTMLElement | null) => {
      if (node) {
        mount(node, content);
      }
    },
  });
}

interface HeroContent {
  logoSrc: string;
  title: string;
  tagline: string;
  primary: HeroLink;
  secondary: HeroAction;
  badges: HeroBadge[];
}

function mountHero(node: HTMLElement, content: HeroContent): void {
  const hero = node as DfkHero;
  hero.setLogo(content.logoSrc);
  hero.setTitle(content.title);
  hero.setTagline(content.tagline);
  hero.setPrimaryAction(content.primary);
  hero.setSecondaryAction(content.secondary);
  hero.setBadges(content.badges);
}

interface FeaturesContent {
  sectionTitle: string;
  items: FeatureItem[];
}

function mountFeatures(node: HTMLElement, content: FeaturesContent): void {
  const features = node as DfkFeatures;
  features.setSectionTitle(content.sectionTitle);
  features.setFeatures(content.items);
}

interface NextStepsContent {
  sectionTitle: string;
  items: NextStepItem[];
}

function mountNextSteps(node: HTMLElement, content: NextStepsContent): void {
  const steps = node as DfkNextSteps;
  steps.setSectionTitle(content.sectionTitle);
  steps.setSteps(content.items);
}

/**
 * Kept out of the JSX below on purpose: a template literal written inline would
 * carry the JSX indentation into the rendered code block. This is the first SQL
 * a reader needs — the community install is a one-time step, `LOAD` is per
 * session.
 */
const INSTALL_SAMPLE = `-- once per installation, then once per session
INSTALL duckfn_open FROM community;
LOAD duckfn_open;`;

/**
 * The other half: the functions the extension registers, called exactly like
 * DuckDB's own. These four are the shapes a first read needs — content becoming
 * a file, content becoming a browser window, an existing path going to the
 * default program, and the one call that starts nothing.
 */
const USAGE_SAMPLE = `SELECT open_write(markdown, 'md');  -- content out to a file, its path back, nothing started
SELECT open_html('<h1>hello</h1>');  -- a temporary .html, opened in the default browser
SELECT open_path('report.html');  -- true once a launcher has been started
SELECT open_commands('report.html');
-- VARCHAR[]: the launch commands open_path would try on this machine, in order`;

/**
 * The shields.io badges ask for `style=flat`, which is the rounded style; the
 * default `flat-square` draws square corners and would clash with the language
 * badges below, which are rounded too. The row has to look like one set, so the
 * shape is decided at the source rather than patched with CSS.
 *
 * These are the facts a reader who only wants to *use* the extension cares
 * about: the latest release, the licence, and which DuckDB versions take it. The
 * Rust toolchain badge the template used to carry is a build requirement and now
 * lives in the development guide instead.
 *
 * 徽章：`<owner>/<repo>` 还没填时（`REPO_URL` 仍是占位符）只显示不依赖仓库地址的那两枚，
 * 免得首页挂着一排坏图。
 */
function badges(repoUrl: string): HeroBadge[] {
  const repoSlug = repoUrl.replace(/^https:\/\/github\.com\//, '');
  const hasRepo = !repoSlug.includes('<');

  return [
    ...(hasRepo
      ? [
          {
            href: `${repoUrl}/releases`,
            src: `https://img.shields.io/github/v/release/${repoSlug}?style=flat`,
            alt: 'Latest release',
          },
        ]
      : []),
    {
      href: `${repoUrl}/blob/main/LICENSE`,
      src: 'https://img.shields.io/badge/license-MIT-14459b.svg?style=flat',
      alt: 'MIT license',
    },
    {
      href: 'https://duckdb.org',
      src: 'https://img.shields.io/badge/DuckDB-1.3%2B-14459b.svg?style=flat',
      alt: 'DuckDB 1.3 or newer',
    },
  ];
}

function heroContent(
  logoSrc: string,
  getStartedHref: string,
  repoUrl: string,
  title: string,
  tagline: string,
): HeroContent {
  return {
    logoSrc,
    title,
    tagline,
    primary: {
      label: translate({id: 'homepage.getStarted', message: 'Get started'}),
      href: getStartedHref,
    },
    secondary: {
      label: translate({
        id: 'homepage.github',
        description: 'Home page button linking to the repository',
        message: 'GitHub',
      }),
      href: repoUrl,
      icon: 'simple-icons:github',
    },
    badges: badges(repoUrl),
  };
}

function featuresContent(): FeaturesContent {
  return {
    sectionTitle: translate({
      id: 'homepage.features.title',
      description: 'Home page section title above the feature cards',
      message: 'What duckfn_open gives you',
    }),
    items: [
      {
        icon: 'lucide:braces',
        title: translate({
          id: 'homepage.features.sql.title',
          description: 'Home page feature card title',
          message: 'Call it from SQL',
        }),
        details: translate({
          id: 'homepage.features.sql.details',
          description: 'Home page feature card description',
          message:
            'Seven function names — open_path, open_write, open_content, open_html, open_print and the rest — are ordinary DuckDB functions: use them in any query, alongside the built-in ones, with nothing to import.',
        }),
      },
      {
        icon: 'lucide:file-output',
        title: translate({
          id: 'homepage.features.content.title',
          description: 'Home page feature card title',
          message: 'Content, not only paths',
        }),
        details: translate({
          id: 'homepage.features.content.details',
          description: 'Home page feature card description',
          message:
            'A report, a chart or a page that has no file yet can be handed over directly: open_write writes it out and returns the path, open_content opens it in the same call, open_html needs no configuration at all, and open_print bypasses the result set for output that is read line by line.',
        }),
      },
      {
        icon: 'lucide:package',
        title: translate({
          id: 'homepage.features.install.title',
          description: 'Home page feature card title',
          message: 'Install in one line',
        }),
        details: translate({
          id: 'homepage.features.install.details',
          description: 'Home page feature card description',
          message:
            'INSTALL duckfn_open FROM community; then LOAD duckfn_open; \u2014 signed builds matched to your DuckDB version and platform, so no -unsigned flag is needed.',
        }),
      },
      {
        icon: 'lucide:monitor-down',
        title: translate({
          id: 'homepage.features.platforms.title',
          description: 'Home page feature card title',
          message: 'Windows, Linux and macOS',
        }),
        details: translate({
          id: 'homepage.features.platforms.details',
          description: 'Home page feature card description',
          message:
            'Each version attaches one .duckdb_extension per desktop platform to a GitHub Release, and LOAD takes a file straight from its URL \u2014 no download step in between. There is no WebAssembly asset: a launcher is desktop software.',
        }),
      },
      {
        icon: 'lucide:shield-check',
        title: translate({
          id: 'homepage.features.versions.title',
          description: 'Home page feature card title',
          message: 'DuckDB 1.3 or newer',
        }),
        details: translate({
          id: 'homepage.features.versions.details',
          description: 'Home page feature card description',
          message:
            "Built against DuckDB's C extension API, so one binary loads into DuckDB 1.3 and later without a recompile.",
        }),
      },
      {
        icon: 'lucide:code',
        title: translate({
          id: 'homepage.features.examples.title',
          description: 'Home page feature card title',
          message: 'Examples to run in your own DuckDB',
        }),
        details: translate({
          id: 'homepage.features.examples.details',
          description: 'Home page feature card description',
          message:
            'The function reference is plain SQL: load the extension into a local DuckDB and paste it in. These pages run nothing for you \u2014 the launcher they talk to lives on the machine the query runs on.',
        }),
      },
      {
        icon: 'lucide:scale',
        title: translate({
          id: 'homepage.features.license.title',
          description: 'Home page feature card title',
          message: 'MIT licensed',
        }),
        details: translate({
          id: 'homepage.features.license.details',
          description: 'Home page feature card description',
          message:
            'Open source under the MIT license: the source, the tests and these pages all live in the repository.',
        }),
      },
    ],
  };
}

function nextStepsContent(
  hrefs: readonly [string, string, string, string],
): NextStepsContent {
  const [intro, installation, functions, development] = hrefs;
  return {
    sectionTitle: translate({
      id: 'homepage.next.title',
      description: 'Home page section title above the link cards',
      message: 'Where to go next',
    }),
    items: [
      {
        href: intro,
        title: translate({
          id: 'homepage.next.intro.title',
          description: 'Home page link card title',
          message: 'Introduction',
        }),
        details: translate({
          id: 'homepage.next.intro.details',
          description: 'Home page link card description',
          message: 'What duckfn_open adds to DuckDB, in one page.',
        }),
      },
      {
        href: installation,
        title: translate({
          id: 'homepage.next.installation.title',
          description: 'Home page link card title',
          message: 'Installation',
        }),
        details: translate({
          id: 'homepage.next.installation.details',
          description: 'Home page link card description',
          message: 'The community repository, a release file, or a local build.',
        }),
      },
      {
        href: functions,
        title: translate({
          id: 'homepage.next.functions.title',
          description: 'Home page link card title',
          message: 'Functions',
        }),
        details: translate({
          id: 'homepage.next.functions.details',
          description: 'Home page link card description',
          message: 'Every function, with an example to run in your own DuckDB.',
        }),
      },
      {
        href: development,
        title: translate({
          id: 'homepage.next.development.title',
          description: 'Home page link card title',
          message: 'Development guide',
        }),
        details: translate({
          id: 'homepage.next.development.details',
          description: 'Home page link card description',
          message: 'Building, testing and releasing the extension from source.',
        }),
      },
    ],
  };
}

function CodeShowcase(): ReactNode {
  return (
    <section className={styles.sectionTint}>
      <div className={styles.sectionInner}>
        <Heading as="h2" className={styles.sectionTitle}>
          <Translate
            id="homepage.showcase.title"
            description="Home page section title above the SQL code blocks">
            Install once, then call it from SQL
          </Translate>
        </Heading>
        <p className={styles.sectionLead}>
          <Translate
            id="homepage.showcase.lead"
            description="Home page paragraph introducing the SQL code blocks">
            The extension adds ordinary SQL functions, with nothing to import at
            the call site. Install it once (left), then use the functions
            anywhere a DuckDB function is allowed (right).
          </Translate>
        </p>
        <div className={styles.codeGrid}>
          <CodeBlock language="sql" title="duckdb">
            {INSTALL_SAMPLE}
          </CodeBlock>
          <div className={styles.codeColumn}>
            <CodeBlock language="sql" title="duckdb">
              {USAGE_SAMPLE}
            </CodeBlock>
            {/* Balances the two columns, and explains the trailing comments. */}
            <p className={styles.codeCaption}>
              <Translate
                id="homepage.showcase.caption"
                description="Home page note under the SQL code block explaining the trailing comments">
                The comments are what each call answers: the first two are the
                content you hand in, the last two are a path that already exists
                — and the final line only lists the commands a launch would try,
                without trying any of them.
              </Translate>
            </p>
          </div>
        </div>
        <p className={styles.showcaseLinkRow}>
          <Link className={styles.showcaseLink} to="/docs/user-guide/functions">
            <Translate
              id="homepage.showcase.link"
              description="Home page link to the function reference">
              The full function reference
            </Translate>
            {/* The official Iconify web component (registered by
                registerDfkElements()); a string `icon` attribute is all it
                needs. createElement keeps it out of the JSX namespace. */}
            {createElement('iconify-icon', {
              icon: 'lucide:arrow-right',
              className: styles.showcaseLinkArrow,
              'aria-hidden': 'true',
            })}
          </Link>
        </p>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  const repoUrl = siteConfig.customFields?.repoUrl as string;
  // Not a hard-coded "/img/...": the site is published under /<repo>/ on GitHub
  // Pages, and only useBaseUrl adds that prefix. The dfk-* components render
  // plain anchors, so every internal href is resolved here before it is passed
  // in.
  const logoUrl = useBaseUrl('img/logo.svg');
  // The hero's primary action is "Get started", so it lands on the install
  // instructions rather than the introduction.
  const getStartedUrl = useBaseUrl('/docs/user-guide/installation');
  const nextHrefs = [
    useBaseUrl('/docs/intro'),
    useBaseUrl('/docs/user-guide/installation'),
    useBaseUrl('/docs/user-guide/functions'),
    useBaseUrl('/docs/development/quick-start'),
  ] as const;

  return (
    <Layout
      title={siteConfig.title}
      description="How to install and use this DuckDB extension, the functions it adds, and where its development guide lives.">
      {/* Layout renders no <main> of its own: this is the page's only one. */}
      <main>
        {dfk(
          'dfk-hero',
          mountHero,
          heroContent(
            logoUrl,
            getStartedUrl,
            repoUrl,
            siteConfig.title,
            translate({id: 'homepage.tagline', message: siteConfig.tagline}),
          ),
        )}
        {dfk('dfk-features', mountFeatures, featuresContent())}
        <CodeShowcase />
        {dfk('dfk-next-steps', mountNextSteps, nextStepsContent(nextHrefs))}
      </main>
    </Layout>
  );
}