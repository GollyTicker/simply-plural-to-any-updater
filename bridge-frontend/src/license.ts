import licenseHtml from '../../docker/license-info-short.txt?raw';

export function renderLicenseInfo() {
  document.querySelector('footer')!.innerHTML = licenseHtml;
}
