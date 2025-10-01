# Tauri Quick Reference

## What is Tauri?

Tauri is a framework for building tiny, fast binaries for all major desktop and mobile platforms. Developers can integrate any frontend framework that compiles to HTML, JavaScript, and CSS for building their user experience while leveraging languages such as Rust, Swift, and Kotlin for backend logic when needed.

Get started building with create-tauri-app by using one of the below commands. Be sure to follow the prerequisites guide to install all of the dependencies required by Tauri. For a more detailed walk through, see Create a Project

```bash
sh <(curl https://create.tauri.app/sh)
```

```bash
# Cargo
cargo install create-tauri-app --locked
cargo create-tauri-app
```

```bash
# npm
npm create tauri-app@latest
```

After you've created your first app, take a look at Project Structure to understand what each file does.

Or explore the project setups and features from the examples (tauri | plugins-workspace)

## Why Tauri?

Tauri has 3 main advantages for developers to build upon:

- Secure foundation for building apps
- Smaller bundle size by using the system's native webview
- Flexibility for developers to use any frontend and bindings for multiple languages

Learn more about the Tauri philosophy in the Tauri 1.0 blog post.

### Secure Foundation

By being built on Rust, Tauri is able to take advantage of the memory, thread, and type-safety offered by Rust. Apps built on Tauri can automatically get those benefits even without needing to be developed by Rust experts.

Tauri also undergoes a security audit for major and minor releases. This not only covers code in the Tauri organization, but also for upstream dependencies that Tauri relies on. Of course this doesn't mitigate all risks, but it provides a solid foundation for developers to build on top of.

Read the Tauri security policy and the Tauri 2.0 audit report.

### Smaller App Size

Tauri apps take advantage of the web view already available on every user's system. A Tauri app only contains the code and assets specific for that app and doesn't need to bundle a browser engine with every app. This means that a minimal Tauri app can be less than 600KB in size.

Learn more about creating optimized apps in the App Size concept.

### Flexible Architecture

Since Tauri uses web technologies that means that virtually any frontend framework is compatible with Tauri. The Frontend Configuration guide contains common configurations for popular frontend frameworks.

Bindings between JavaScript and Rust are available to developers using the invoke function in JavaScript and Swift and Kotlin bindings are available for Tauri Plugins.

TAO is responsible for Tauri window creation and WRY is responsible for web view rendering. These are libraries maintained by Tauri and can be consumed directly if deeper system integration is required outside of what Tauri exposes.

In addition, Tauri maintains a number of plugins to extend what core Tauri exposes. You can find those plugins alongside those provided by the community in the Plugins section.

---

## Prerequisites

In order to get started building your project with Tauri you'll first need to install a few dependencies:

- System Dependencies
- Rust
- Configure for Mobile Targets (only required if developing for mobile)

### System Dependencies

Follow the link to get started for your respective operating system:

- Linux (see below for specific distributions)
- macOS Catalina (10.15) and later
- Windows 7 and later

#### Linux

Tauri requires various system dependencies for development on Linux. These may be different depending on your distribution but we've included some popular distributions below to help you get setup.

##### Fedora

```bash
sudo dnf check-update
sudo dnf install webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  libxdo-devel
sudo dnf group install "c-development"
```

#### Windows

Tauri uses the Microsoft C++ Build Tools for development as well as Microsoft Edge WebView2. These are both required for development on Windows.

##### Microsoft C++ Build Tools

- Download the Microsoft C++ Build Tools installer and open it to begin installation.
- During installation check the "Desktop development with C++" option.

Next: Install WebView2.

##### WebView2

> Tip
>
> WebView2 is already installed on Windows 10 (from version 1803 onward) and later versions of Windows. If you are developing on one of these versions then you can skip this step and go directly to installing Rust.

Tauri uses Microsoft Edge WebView2 to render content on Windows.

Install WebView2 by visiting the WebView2 Runtime download section. Download the "Evergreen Bootstrapper" and install it.

Next: Check VBSCRIPT.

##### VBSCRIPT (for MSI installers)

MSI package building only.

This is only required if you plan to build MSI installer packages ("targets": "msi" or "targets": "all" in `tauri.conf.json`).

Building MSI packages on Windows requires the VBSCRIPT optional feature to be enabled. This feature is enabled by default on most Windows installations, but may have been disabled on some systems.

If you encounter errors like failed to run `light.exe` when building MSI packages, you may need to enable the VBSCRIPT feature:

- Open Settings → Apps → Optional features → More Windows features
- Locate VBSCRIPT in the list and ensure it’s checked
- Click Next and restart your computer if prompted

Note: VBSCRIPT is currently enabled by default on most Windows installations, but is being deprecated and may be disabled in future Windows versions.

### Rust

Tauri is built with Rust and requires it for development. Install Rust using one of following methods. You can view more installation methods at https://www.rust-lang.org/tools/install.

#### Linux and macOS

Install via rustup using the following command:

```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

> **Security Tip**
> 
> We have audited this bash script, and it does what it says it is supposed to do. Nevertheless, before blindly curl-bashing a script, it is always wise to look at it first.
> 
> Here is the file as a plain script: rustup.sh

Be sure to restart your Terminal (and in some cases your system) for the changes to take affect.

#### Windows

Visit https://www.rust-lang.org/tools/install to install rustup.

Alternatively, you can use winget to install rustup using the following command in PowerShell:

```bash
winget install --id Rustlang.Rustup
```

**MSVC toolchain as default**

For full support for Tauri and tools like trunk make sure the MSVC Rust toolchain is the selected default host triple in the installer dialog. Depending on your system it should be either `x86_64-pc-windows-msvc`, `i686-pc-windows-msvc`, or `aarch64-pc-windows-msvc`.

If you already have Rust installed, you can make sure the correct toolchain is installed by running this command:

```bash
rustup default stable-msvc
```

Be sure to restart your Terminal (and in some cases your system) for the changes to take affect.

### Node.js

> **JavaScript ecosystem**
> 
> Only if you intend to use a JavaScript frontend framework

Go to the Node.js website, download the Long Term Support (LTS) version and install it.
Check if Node was successfully installed by running:

```bash
node -v
# v20.10.0
npm -v
# 10.2.3
```

It's important to restart your Terminal to ensure it recognizes the new installation. In some cases, you might need to restart your computer.

While npm is the default package manager for Node.js, you can also use others like pnpm or yarn. To enable these, run `corepack enable` in your Terminal. This step is optional and only needed if you prefer using a package manager other than npm.

---

## Create a Project

One thing that makes Tauri so flexible is its ability to work with virtually any frontend framework. We've created the create-tauri-app utility to help you create a new Tauri project using one of the officially maintained framework templates.

create-tauri-app currently includes templates for vanilla (HTML, CSS and JavaScript without a framework), Vue.js, Svelte, React, SolidJS, Angular, Preact, Yew, Leptos, and Sycamore. You can also find or add your own community templates and frameworks in the Awesome Tauri repo.

Alternatively, you can add Tauri to an existing project to quickly turn your existing codebase into a Tauri app.

### Using create-tauri-app

Follow along with the prompts to choose your project name, frontend language, package manager, and frontend framework, and frontend framework options if applicable.

> **Not sure what to choose?**
> 
> We recommend starting with the vanilla template (HTML, CSS, and JavaScript without a frontend framework) to get started. You can always integrate a frontend framework later.

#### Scaffold a new project

Choose a name and a bundle identifier (unique-id for your app):

```
? Project name (tauri-app) ›
? Identifier (com.tauri-app.app) ›
```

Select a flavor for your frontend. First the language:

```
? Choose which language to use for your frontend ›
❯ Rust  (cargo)
  TypeScript / JavaScript  (pnpm, yarn, npm, bun)
  .NET  (dotnet)
```

Select a package manager (if there are multiple available):

**Options for TypeScript / JavaScript:**

```
? Choose your package manager ›
❯ pnpm
  yarn
  npm
  bun
```

Select a UI Template and flavor (if there are multiple available):

**Options for Rust:**

```
? Choose your UI template ›
❯ Vanilla
  Yew
  Leptos
  Sycamore
```

**Options for TypeScript / JavaScript:**

```
? Choose your UI template ›
❯ Vanilla
  Vue
  Svelte
  React
  Solid
  Angular
  Preact
```

```
? Choose your UI flavor ›
❯ TypeScript
  JavaScript
```

**Options for .NET:**

```
? Choose your UI template ›
❯ Blazor  (https://dotnet.microsoft.com/en-us/apps/aspnet/web-apps/blazor/)
```

Once completed, the utility reports that the template has been created and displays how to run it using the configured package manager. If it detects missing dependencies on your system, it prints a list of packages and prompts how to install them.

### Start the development server

After create-tauri-app has completed, you can navigate into your project's folder, install dependencies, and then use the Tauri CLI to start the development server:

```bash
# npm
cd tauri-app
npm install
npm run tauri dev
```

```bash
# cargo
cd tauri-app
cargo tauri dev
```

You'll now see a new window open with your app running.

**Congratulations! You've made your Tauri app! 🚀**

### Manual Setup (Tauri CLI)

If you already have an existing frontend or prefer to set it up yourself, you can use the Tauri CLI to initialize the backend for your project separately.

> **Note**
> 
> The following example assumes you are creating a new project. If you've already initialized the frontend of your application, you can skip the first step.

Create a new directory for your project and initialize the frontend. You can use plain HTML, CSS, and JavaScript, or any framework you prefer such as Next.js, Nuxt, Svelte, Yew, or Leptos. You just need a way of serving the app in your browser. Just as an example, this is how you would setup a simple Vite app:

```bash
# npm
mkdir tauri-app
cd tauri-app
npm create vite@latest .
```

Then, install Tauri's CLI tool using your package manager of choice. If you are using cargo to install the Tauri CLI, you will have to install it globally.

```bash
# npm
npm install -D @tauri-apps/cli@latest
```

```bash
# cargo
cargo install tauri-cli --version "^2.0.0" --locked
```

Determine the URL of your frontend development server. This is the URL that Tauri will use to load your content. For example, if you are using Vite, the default URL is `http://localhost:5173`.

In your project directory, initialize Tauri:

```bash
# npm
npx tauri init
```

```bash
# cargo
cargo tauri init
```

After running the command it will display a prompt asking you for different options:

```
✔ What is your app name? tauri-app
✔ What should the window title be? tauri-app
✔ Where are your web assets located? ..
✔ What is the url of your dev server? http://localhost:5173
✔ What is your frontend dev command? pnpm run dev
✔ What is your frontend build command? pnpm run build
```

This will create a `src-tauri` directory in your project with the necessary Tauri configuration files.

Verify your Tauri app is working by running the development server:

```bash
# npm
npx tauri dev
```

```bash
# cargo
cargo tauri dev
```

This command will compile the Rust code and open a window with your web content.

**Congratulations! You've created a new Tauri project using the Tauri CLI! 🚀**

---

## Project Structure

A Tauri project is usually made of 2 parts, a Rust project and a JavaScript project (optional), and typically the setup looks something like this:

```
.
├── package.json
├── index.html
├── src/
│   ├── main.js
├── src-tauri/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   ├── icons/
│   │   ├── icon.png
│   │   ├── icon.icns
│   │   └── icon.ico
│   └── capabilities/
│       └── default.json
```

In this case, the JavaScript project is at the top level, and the Rust project is inside `src-tauri/`, the Rust project is a normal Cargo project with some extra files:

- `tauri.conf.json` is the main configuration file for Tauri, it contains everything from the application identifier to dev server url, this file is also a marker for the Tauri CLI to find the Rust project, to learn more about it, see Tauri Config
- `capabilities/` directory is the default folder Tauri reads capability files from (in short, you need to allow commands here to use them in your JavaScript code), to learn more about it, see Security
- `icons/` directory is the default output directory of the tauri icon command, it's usually referenced in `tauri.conf.json > bundle > icon` and used for the app's icons
- `build.rs` contains `tauri_build::build()` which is used for tauri's build system
- `src/lib.rs` contains the Rust code and the mobile entry point (the function marked with `#[cfg_attr(mobile, tauri::mobile_entry_point)]`), the reason we don't write directly in main.rs is because we compile your app to a library in mobile builds and load them through the platform frameworks
- `src/main.rs` is the main entry point for the desktop, and we run `tauri_app_lib::run()` in main to use the same entry point as mobile, so to keep it simple, don't modify this file, modify lib.rs instead

Tauri works similar to a static web host, and the way it builds is that you would compile your JavaScript project to static files first, and then compile the Rust project that will bundle those static files in, so the JavaScript project setup is basically the same as if you were to build a static website, to learn more, see Frontend Configuration

If you want to work with Rust code only, simply remove everything else and use the `src-tauri/` folder as your top level project or as a member of your Rust workspace

---

## SvelteKit

SvelteKit is a meta framework for Svelte. Learn more about SvelteKit at https://svelte.dev/. This guide is accurate as of SvelteKit 2.20.4 / Svelte 5.25.8.

### Checklist

- Use SSG and SPA via static-adapter. Tauri doesn't support server-based solutions.
- If using SSG with prerendering, be aware that load functions will not have access to tauri APIs during the build process of your app. Using SPA mode (without prerendering) is recommended since the load functions will only run in the webview with access to tauri APIs.
- Use `build/` as frontendDist in `tauri.conf.json`.

### Example Configuration

#### Install @sveltejs/adapter-static

```bash
npm install --save-dev @sveltejs/adapter-static
```

#### Update Tauri configuration

```json
// tauri.conf.json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../build"
  }
}
```

#### Update SvelteKit configuration

```javascript
// svelte.config.js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://svelte.dev/docs/kit/integrations#preprocessors
  // for more information about preprocessors
  preprocess: vitePreprocess(),

  kit: {
    adapter: adapter({
      fallback: 'index.html',
    }),
  },
};

export default config;
```

#### Disable SSR

Lastly, we need to disable SSR by adding a root `+layout.ts` file (or `+layout.js` if you are not using TypeScript) with these contents:

```typescript
// src/routes/+layout.ts
export const ssr = false;
```

Note that static-adapter doesn't require you to disable SSR for the whole app but it makes it possible to use APIs that depend on the global window object (like Tauri's API) without Client-side checks.

Furthermore, if you prefer Static Site Generation (SSG) over Single-Page Application (SPA) mode, you can change the adapter configurations and `+layout.ts` according to the adapter docs.

---

## Vite

Vite is a build tool that aims to provide a faster and leaner development experience for modern web projects. This guide is accurate as of Vite 5.4.8.

### Checklist

- Use `../dist` as frontendDist in `src-tauri/tauri.conf.json`.
- Use `process.env.TAURI_DEV_HOST` as the development server host IP when set to run on iOS physical devices.

### Example configuration

#### Update Tauri configuration

Assuming you have the following dev and build scripts in your `package.json`:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  }
}
```

You can configure the Tauri CLI to use your Vite development server and dist folder along with the hooks to automatically run the Vite scripts:

```json
// tauri.conf.json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  }
}
```

#### Update Vite configuration

```javascript
// vite.config.js
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  // prevent vite from obscuring rust errors
  clearScreen: false,
  server: {
    // make sure this port matches the devUrl port in tauri.conf.json file
    port: 5173,
    // Tauri expects a fixed port, fail if that port is not available
    strictPort: true,
    // if the host Tauri is expecting is set, use it
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,

    watch: {
      // tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
  // Env variables starting with the item of `envPrefix` will be exposed in tauri's source code through `import.meta.env`.
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target:
      process.env.TAURI_ENV_PLATFORM == 'windows'
        ? 'chrome105'
        : 'safari13',
    // don't minify for debug builds
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
```

---

## GitHub (Windows Builds)

This guide will show you how to use tauri-action in GitHub Actions to easily build and upload your app, and how to make Tauri’s updater query the newly created GitHub release for updates.

Lastly, it will also show how to set up a more complicated build pipeline for Linux Arm AppImages.

### Getting Started

To set up tauri-action you must first set up a GitHub repository. You can also use this action on a repository that does not have Tauri configured yet since it can automatically initialize Tauri for you, please see the action’s readme for necessary configuration options.

Go to the Actions tab on your GitHub project page and select “New workflow”, then choose “Set up a workflow yourself”. Replace the file with the workflow from below or from one of the action’s examples.

### Configuration

Please see the tauri-action readme for all available configuration options.

When your app is not on the root of the repository, use the projectPath input.

You may freely modify the workflow name, change its triggers, and add more steps such as npm run lint or npm run test. The important part is that you keep the below line at the end of the workflow since this runs the build script and releases your app.

### How to Trigger

The release workflow shown below and in the tauri-action examples is triggered by pushed to the release branch. The action automatically creates a git tag and a title for the GitHub release using the application version.

As another example, you can also change the trigger to run the workflow on the push of a version git tag such as app-v0.7.0:

```yml
name: 'publish'

on:
  push:
    tags:
      - 'app-v*'
```

For a full list of possible trigger configurations, check out the official GitHub documentation.

### Example Workflow

Below is an example workflow that has been set up to run every time you push to the release branch.

This workflow will build and release your app for Windows x64, Linux x64, Linux Arm64, macOS x64 and macOS Arm64 (M1 and above).

The steps this workflow takes are:

- Checkout the repository using actions/checkout@v4.
- Install Linux system dependencies required to build the app.
- Set up Node.js LTS and a cache for global npm/yarn/pnpm package data using actions/setup-node@v4.
- Set up Rust and a cache for Rust’s build artifacts using dtolnay/rust-toolchain@stable and swatinem/rust-cache@v2.
- Install the frontend dependencies and, if not configured as beforeBuildCommand, run the web app’s build script.
- Lastly, it uses tauri-apps/tauri-action@v0 to run tauri build, generate the artifacts, and create a GitHub release.

```yml
name: 'publish'

on:
  workflow_dispatch:
  push:
    branches:
      - release

jobs:
  publish-tauri:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: 'macos-latest' # for Arm based macs (M1 and above).
            args: '--target aarch64-apple-darwin'
          - platform: 'macos-latest' # for Intel based macs.
            args: '--target x86_64-apple-darwin'
          - platform: 'ubuntu-22.04'
            args: ''
          - platform: 'ubuntu-22.04-arm' # Only available in public repos.
            args: ''
          - platform: 'windows-latest'
            args: ''

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: install dependencies (ubuntu only)
        if: matrix.platform == 'ubuntu-22.04' # This must match the platform value defined above.
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

      - name: setup node
        uses: actions/setup-node@v4
        with:
          node-version: lts/*
          cache: 'yarn' # Set this to npm, yarn or pnpm.

      - name: install Rust stable
        uses: dtolnay/rust-toolchain@stable # Set this to dtolnay/rust-toolchain@nightly
        with:
          # Those targets are only used on macos runners so it's in an `if` to slightly speed up windows and linux builds.
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'

      - name: install frontend dependencies
        # If you don't have `beforeBuildCommand` configured you may want to build your frontend here too.
        run: yarn install # change this to npm or pnpm depending on which one you use.

      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: app-v__VERSION__ # the action automatically replaces __VERSION__ with the app version.
          releaseName: 'App v__VERSION__'
          releaseBody: 'See the assets to download this version and install.'
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.args }}
```

For more configuration options, check out the tauri-action repository and its examples.

### Troubleshooting

#### GitHub Environment Token

The GitHub Token is automatically issued by GitHub for each workflow run without further configuration, which means there is no risk of secret leakage. This token however only has read permissions by default and you may get a “Resource not accessible by integration” error when running the workflow. If this happens, you may need to add write permissions to this token. To do this, go to your GitHub project settings, select Actions, scroll down to Workflow permissions, and check “Read and write permissions”.

You can see the GitHub Token being passed to the workflow via this line in the workflow:

```yml
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## WebDriver Testing

WebDriver is a standardized interface to interact with web documents primarily intended for automated testing. Tauri supports the WebDriver interface by leveraging the native platform's WebDriver server underneath a cross-platform wrapper `tauri-driver`.

On desktop, only Windows and Linux are supported due to macOS not having a WKWebView driver tool available. iOS and Android work through Appium 2, but the process is not currently streamlined.

### System Dependencies

Install the latest `tauri-driver` or update an existing installation by running:

```bash
cargo install tauri-driver --locked
```

Because we currently utilize the platform's native WebDriver server, there are some requirements for running `tauri-driver` on supported platforms.

#### Linux

We use WebKitWebDriver on Linux platforms. Check if this binary exists already by running the `which WebKitWebDriver` command as some distributions bundle it with the regular WebKit package. Other platforms may have a separate package for them, such as `webkit2gtk-driver` on Debian-based distributions.

### WebdriverIO Setup

> **Note**
> 
> Make sure to go through the prerequisites instructions to be able to follow this guide.

This WebDriver testing example will use WebdriverIO, and its testing suite. It is expected to have Node.js already installed, along with npm or yarn although the finished example project uses pnpm.

#### Create a Directory for the Tests

Let's create a space to write these tests in our project. We will be using a nested directory for this example project as we will later also go over other frameworks, but typically you only need to use one. Create the directory we will use with `mkdir e2e-tests`. The rest of this guide assumes you are inside the `e2e-tests` directory.

#### Initializing a WebdriverIO Project

We will be using a pre-existing `package.json` to bootstrap this test suite because we have already chosen specific WebdriverIO config options and want to showcase a simple working solution. The bottom of this section has a collapsed guide on setting it up from scratch.

**package.json:**

```json
{
  "name": "webdriverio",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "test": "wdio run wdio.conf.js"
  },
  "dependencies": {
    "@wdio/cli": "^9.19.0"
  },
  "devDependencies": {
    "@wdio/local-runner": "^9.19.0",
    "@wdio/mocha-framework": "^9.19.0",
    "@wdio/spec-reporter": "^9.19.0"
  }
}
```

We have a script that runs a WebdriverIO config as a test suite exposed as the test command. We also have various dependencies added by the `@wdio/cli` command when we first set it up. In short, these dependencies are for the most simple setup using a local WebDriver runner, Mocha as the test framework, and a simple Spec Reporter.

#### Configuration

You may have noticed that the test script in our `package.json` mentions a file `wdio.conf.js`. That's the WebdriverIO config file which controls most aspects of our testing suite.

**wdio.conf.js:**

```javascript
import os from 'os';
import path from 'path';
import { spawn, spawnSync } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

// keep track of the `tauri-driver` child process
let tauriDriver;
let exit = false;

export const config = {
  host: '127.0.0.1',
  port: 4444,
  specs: ['./test/specs/**/*.js'],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: '../src-tauri/target/debug/tauri-app',
      },
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  // ensure the rust project is built since we expect this binary to exist for the webdriver sessions
  onPrepare: () => {
    spawnSync('yarn', ['tauri', 'build', '--debug', '--no-bundle'], {
      cwd: path.resolve(__dirname, '..'),
      stdio: 'inherit',
      shell: true,
    });
  },

  // ensure we are running `tauri-driver` before the session starts so that we can proxy the webdriver requests
  beforeSession: () => {
    tauriDriver = spawn(
      path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
      [],
      { stdio: [null, process.stdout, process.stderr] }
    );

    tauriDriver.on('error', (error) => {
      console.error('tauri-driver error:', error);
      process.exit(1);
    });
    tauriDriver.on('exit', (code) => {
      if (!exit) {
        console.error('tauri-driver exited with code:', code);
        process.exit(1);
      }
    });
  },

  // clean up the `tauri-driver` process we spawned at the start of the session
  // note that afterSession might not run if the session fails to start, so we also run the cleanup on shutdown
  afterSession: () => {
    closeTauriDriver();
  },
};

function closeTauriDriver() {
  exit = true;
  tauriDriver?.kill();
}

function onShutdown(fn) {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };

  process.on('exit', cleanup);
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('SIGHUP', cleanup);
  process.on('SIGBREAK', cleanup);
}

// ensure tauri-driver is closed when our test process exits
onShutdown(() => {
  closeTauriDriver();
});
```

If you are interested in the properties on the config object, we suggest reading the documentation. For non-WDIO specific items, there are comments explaining why we are running commands in `onPrepare`, `beforeSession`, and `afterSession`. We also have our specs set to `"./test/specs/**/*.js"`, so let's create a spec now.

#### Test Specs

A spec contains the code that is testing your actual application. The test runner will load these specs and automatically run them as it sees fit. Let's create our spec now in the directory we specified.

**test/specs/example.e2e.js:**

```javascript
// calculates the luma from a hex color `#abcdef`
function luma(hex) {
  if (hex.startsWith('#')) {
    hex = hex.substring(1);
  }

  const rgb = parseInt(hex, 16);
  const r = (rgb >> 16) & 0xff;
  const g = (rgb >> 8) & 0xff;
  const b = (rgb >> 0) & 0xff;
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

describe('Hello Tauri', () => {
  it('should be cordial', async () => {
    const header = await $('body > h1');
    const text = await header.getText();
    expect(text).toMatch(/^[hH]ello/);
  });

  it('should be excited', async () => {
    const header = await $('body > h1');
    const text = await header.getText();
    expect(text).toMatch(/!$/);
  });

  it('should be easy on the eyes', async () => {
    const body = await $('body');
    const backgroundColor = await body.getCSSProperty('background-color');
    expect(luma(backgroundColor.parsed.hex)).toBeLessThan(100);
  });
});
```

The `luma` function on top is just a helper function for one of our tests and is not related to the actual testing of the application. If you are familiar with other testing frameworks, you may notice similar functions being exposed that are used, such as `describe`, `it`, and `expect`. The other APIs, such as items like `$` and its exposed methods, are covered by the [WebdriverIO API docs](https://webdriver.io/docs/api).

#### Running the Test Suite

Now that we are all set up with config and a spec let's run it!

```bash
# npm
npm test

# yarn  
yarn test
```

We should see output the following output:

```
➜  webdriverio git:(main) ✗ yarn test
yarn run v1.22.11
$ wdio run wdio.conf.js

Execution of 1 workers started at 2021-08-17T08:06:10.279Z

[0-0] RUNNING in undefined - /test/specs/example.e2e.js
[0-0] PASSED in undefined - /test/specs/example.e2e.js

"spec" Reporter:
------------------------------------------------------------------
[wry 0.12.1 linux #0-0] Running: wry(v0.12.1) on linux
[wry 0.12.1 linux #0-0] Session ID: 81e0107b-4d38-4eed-9b10-ee80ca47bb83
[wry 0.12.1 linux #0-0]
[wry 0.12.1 linux #0-0] » /test/specs/example.e2e.js
[wry 0.12.1 linux #0-0] Hello Tauri
[wry 0.12.1 linux #0-0]    ✓ should be cordial
[wry 0.12.1 linux #0-0]    ✓ should be excited
[wry 0.12.1 linux #0-0]    ✓ should be easy on the eyes
[wry 0.12.1 linux #0-0]
[wry 0.12.1 linux #0-0] 3 passing (244ms)

Spec Files: 1 passed, 1 total (100% completed) in 00:00:01

Done in 1.98s.
```

We see the Spec Reporter tell us that all 3 tests from the `test/specs/example.e2e.js` file, along with the final report `Spec Files: 1 passed, 1 total (100% completed) in 00:00:01`.

Using the WebdriverIO test suite, we just easily enabled e2e testing for our Tauri application from just a few lines of configuration and a single command to run it! Even better, we didn't have to modify the application at all.