const { notarize } = require('@electron/notarize');
const path = require('path');

exports.default = async function notarizing(context) {
  const { electronPlatformName, appOutDir } = context;
  if (electronPlatformName !== 'darwin') {
    return;
  }

  const appName = context.packager.appInfo.productFilename;
  const appleId = process.env.APPLE_ID;
  const appleIdPassword = process.env.APPLE_ID_PASSWORD;
  const teamId = process.env.APPLE_TEAM_ID;

  if (!appleId || !appleIdPassword || !teamId) {
    console.warn('Skipping macOS notarization: Missing APPLE_ID, APPLE_ID_PASSWORD, or APPLE_TEAM_ID environment variables.');
    return;
  }

  console.log(`Notarizing ${appName}...`);

  return await notarize({
    appPath: path.join(appOutDir, `${appName}.app`),
    appleId,
    appleIdPassword,
    teamId,
  });
};
