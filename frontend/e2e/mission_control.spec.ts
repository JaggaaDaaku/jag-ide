import { test, expect } from '@playwright/test';

test.describe('Mission Control', () => {
  test('should display active agents and system health', async ({ page }) => {
    await page.goto('/');
    
    // Welcome screen should be visible initially
    await expect(page.locator('.welcome-title')).toContainText('Jag IDE');
    
    // Enter Mission Control
    await page.click('button:has-text("Start Mission Control")');
    
    // Check for core dashboard elements
    await expect(page.locator('h1')).toContainText('Jag Mission Control');
    
    // Check for agent status grid (there are 4 agents defined in the component)
    const agentCards = page.locator('.agent-card');
    await expect(agentCards).toHaveCount(4);
  });

  test('should allow starting a new mission', async ({ page }) => {
    await page.goto('/');
    
    // Enter Mission Control
    await page.click('button:has-text("Start Mission Control")');
    
    const startMissionBtn = page.locator('button:has-text("Start Mission")');
    await expect(startMissionBtn).toBeVisible();
    
    await page.fill('input[placeholder*="Tell Jag what to build"]', 'Test E2E Mission');
    await startMissionBtn.click();
    
    // Should show the workflow timeline or active mission state
    await expect(page.locator('h2:has-text("Workflow Timeline")')).toBeVisible();
  });
});
