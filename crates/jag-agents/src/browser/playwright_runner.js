// File: crates/jag-agents/src/browser/playwright_runner.js
const { chromium } = require('playwright');

/**
 * Expected JSON input schema:
 * {
 *   "action": "screenshot" | "click" | "fill",
 *   "url": "string",
 *   "selector": "string" (optional),
 *   "path": "string" (optional, for screenshots),
 *   "timeout_ms": number (optional),
 *   "headless": boolean,
 *   "viewport": { "width": number, "height": number }
 * }
 */

async function run() {
    const inputJson = process.argv[2];
    if (!inputJson) {
        console.error("Missing command JSON argument");
        process.exit(1);
    }

    const cmd = JSON.parse(inputJson);
    const headless = cmd.headless; 
    const viewport = cmd.viewport || { width: 1280, height: 720 };

    let browser;
    try {
        browser = await chromium.launch({ 
            headless: headless
        });
        const context = await browser.newContext({
            viewport: viewport
        });
        const page = await context.newPage();

        let response = { success: true, data: null };

        switch (cmd.action) {
            case 'screenshot':
                if (!cmd.url) throw new Error("Missing URL for screenshot");
                await page.goto(cmd.url, { waitUntil: 'networkidle', timeout: cmd.timeout_ms || 30000 });
                
                if (cmd.selector) {
                    const element = await page.waitForSelector(cmd.selector, { timeout: 10000 });
                    await element.screenshot({ path: cmd.path });
                } else {
                    await page.screenshot({ path: cmd.path, fullPage: true });
                }
                break;

            case 'click':
                if (!cmd.url) throw new Error("Missing URL for click");
                await page.goto(cmd.url, { waitUntil: 'networkidle', timeout: cmd.timeout_ms || 30000 });
                await page.click(cmd.selector, { timeout: 10000 });
                break;

            case 'fill':
                if (!cmd.url) throw new Error("Missing URL for fill");
                await page.goto(cmd.url, { waitUntil: 'networkidle', timeout: cmd.timeout_ms || 30000 });
                await page.fill(cmd.selector, cmd.data, { timeout: 10000 });
                break;

            default:
                throw new Error(`Unknown action: ${cmd.action}`);
        }

        console.log(JSON.stringify(response));
    } catch (err) {
        console.log(JSON.stringify({ 
            success: false, 
            error: err.message 
        }));
    } finally {
        if (browser) await browser.close();
    }
}

run();
