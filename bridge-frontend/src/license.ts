import licenseHtml from '../../docker/license-info-short.txt?raw'
import { SP2ANY_VERSION } from './sp2any.bindings'

export function renderLicenseInfo() {
  document.querySelector('footer')!.innerHTML = SP2ANY_VERSION + ' | ' + licenseHtml
}
