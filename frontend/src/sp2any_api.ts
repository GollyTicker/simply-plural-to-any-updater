import axios, { AxiosError } from 'axios'
import type {
  SP2AnyVariantInfo,
  JwtString,
  UserConfigDbEntries,
  UserLoginCredentials,
  UserUpdatersStatuses,
  VRChatCredentials,
  VRChatCredentialsWithCookie,
  VRChatCredentialsWithTwoFactorAuth,
  VRChatAuthResponse,
} from './sp2any.bindings'
import { getJwt, logoutAndBackToStart, setJwt } from './jwt'
import router from './router'

export const http = axios.create({
  baseURL: import.meta.env.VITE_SP2ANY_BASE_URL || '' /* use relate url by default */,
})

export function detailed_error_string(error: any): string {
  const axiosErrorString = (<AxiosError>error)?.response?.data ?? ''
  return error.toString() + '. ' + axiosErrorString
}

http.interceptors.response.use(
  (response) => response,
  (error) => {
    const isLoginAttempt = router.currentRoute.value.path === '/login'
    if (error.response && [401, 403].includes(error.response.status) && !isLoginAttempt) {
      console.warn('Auth failed with 401/403 on request. Now redirecting to login. Error:', error)
      logoutAndBackToStart()
    }
    return Promise.reject(error)
  },
)

// Also handles the storage of the JwtString
export const sp2any_api = {
  login: async function (creds: UserLoginCredentials): Promise<JwtString> {
    const jwtString = await http.post<JwtString>('/api/user/login', creds)
    setJwt(jwtString.data)
    return jwtString.data
  },
  register: async function (creds: UserLoginCredentials): Promise<void> {
    await http.post('/api/user/register', creds)
  },
  get_updater_status: async function (): Promise<UserUpdatersStatuses> {
    const jwtString = await getJwt()
    const response = await http.get<UserUpdatersStatuses>('/api/updaters/status', {
      headers: { Authorization: `Bearer ${jwtString.inner}` },
    })
    return response.data
  },
  get_config: async function (): Promise<UserConfigDbEntries> {
    const jwtString = await getJwt()
    const response = await http.get<UserConfigDbEntries>('/api/user/config', {
      headers: { Authorization: `Bearer ${jwtString.inner}` },
    })
    return response.data
  },
  get_defaults: async function (): Promise<UserConfigDbEntries> {
    const response = await http.get<UserConfigDbEntries>('/api/config/defaults')
    return response.data
  },
  set_config_and_restart: async function (config: UserConfigDbEntries): Promise<void> {
    const jwtString = await getJwt()
    await http.post('/api/user/config_and_restart', config, {
      headers: { Authorization: `Bearer ${jwtString.inner}` },
    })
  },
  vrchat_request_2fa: async function (creds: VRChatCredentials): Promise<VRChatAuthResponse> {
    const jwtString = await getJwt()
    const response = await http.post<VRChatAuthResponse>(
      '/api/user/platform/vrchat/auth_2fa/request',
      creds,
      { headers: { Authorization: `Bearer ${jwtString.inner}` } },
    )
    return response.data
  },
  vrchat_resolve_2fa: async function (
    creds_with_tfa: VRChatCredentialsWithTwoFactorAuth,
  ): Promise<VRChatCredentialsWithCookie> {
    const jwtString = await getJwt()
    const response = await http.post<VRChatCredentialsWithCookie>(
      '/api/user/platform/vrchat/auth_2fa/resolve',
      creds_with_tfa,
      { headers: { Authorization: `Bearer ${jwtString.inner}` } },
    )
    return response.data
  },
  get_variant_info: async function (): Promise<SP2AnyVariantInfo> {
    const response = await http.get<SP2AnyVariantInfo>('/api/meta/sp2any-variant-info')
    return response.data
  },
}
