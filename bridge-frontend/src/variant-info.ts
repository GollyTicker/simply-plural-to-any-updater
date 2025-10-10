import { invoke } from '@tauri-apps/api/core'
import { type SP2AnyVariantInfo } from './sp2any.bindings'

export async function fetchAndRenderVariantInfo(): Promise<[string, SP2AnyVariantInfo]> {
  console.log('fetch_base_url_and_variant_info ...')
  let result = await invoke<[string, SP2AnyVariantInfo]>('fetch_base_url_and_variant_info')
  console.log('fetch_base_url_and_variant_info.0:', result[0])
  console.log('fetch_base_url_and_variant_info.1:', result[1])

  let bridgeVersion = await getBridgeVersion()
  console.log('bridgeVersion', bridgeVersion)

  let element = document.querySelector<HTMLParagraphElement>('#variant-info')!

  let { show_in_ui, variant, description } = result[1]

  element.style.display = show_in_ui ? 'inline-block' : 'none'
  element.innerText = '@' + variant
  if (description) {
    element.title = description
  }

  if (bridgeVersion !== result[1].version) {
    document.querySelector<HTMLDivElement>('#update-bridge-note')!.innerText =
      '⚠️ SP2Any-Bridge is outdated. Install the newest version from SP2Any Website! ⚠️'
  }

  return result
}

async function getBridgeVersion(): Promise<string> {
  return await invoke<string>('get_bridge_version')
}
