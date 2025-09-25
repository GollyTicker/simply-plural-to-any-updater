import axios from 'axios';
import type { JwtString, UserConfigDbEntries, UserLoginCredentials, UserUpdatersStatuses, VRChatCredentials, VRChatCredentialsWithCookie, TwoFactorCodeRequiredResponse, VRChatCredentialsWithTwoFactorAuth, VrchatAuthResponse } from './sp2any.bindings';
import router from './router'

export const http = axios.create({
  baseURL: 'http://localhost:8080',
});

http.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response && [401, 403].includes(error.response.status)) {
      console.warn("Auth failed with 401/403 on request. Now redirecting to login. Error:", error);
      localStorage.removeItem('jwt');
      router.push('/login');
    }
    return Promise.reject(error);
  }
);

// Also handles the storage of the JwtString
export const sp2any_api = {
  login: async function (creds: UserLoginCredentials): Promise<JwtString> {
    let jwtString = await http.post<JwtString>('/api/user/login', creds);
    localStorage.setItem("jwt", JSON.stringify(jwtString.data));
    return jwtString.data;
  },
  get_updater_status: async function (): Promise<UserUpdatersStatuses> {
    let jwtString: JwtString = JSON.parse(localStorage.getItem("jwt")!);
    let response = await http.get<UserUpdatersStatuses>('/api/updaters/status', { headers: { Authorization: `Bearer ${jwtString.inner}` } });
    return response.data;
  },
  get_config: async function (): Promise<UserConfigDbEntries> {
    let jwtString: JwtString = JSON.parse(localStorage.getItem("jwt")!);
    let response = await http.get<UserConfigDbEntries>('/api/user/config', { headers: { Authorization: `Bearer ${jwtString.inner}` } });
    return response.data;
  },
  set_config_and_restart: async function (config: UserConfigDbEntries): Promise<void> {
    let jwtString: JwtString = JSON.parse(localStorage.getItem("jwt")!);
    await http.post('/api/user/config_and_restart', config, { headers: { Authorization: `Bearer ${jwtString.inner}` } });
  },
  vrchat_request_2fa: async function (creds: VRChatCredentials): Promise<VrchatAuthResponse> {
    let jwtString: JwtString = JSON.parse(localStorage.getItem("jwt")!);
    let response = await http.post<VrchatAuthResponse>('/api/user/platform/vrchat/auth_2fa/request', creds, { headers: { Authorization: `Bearer ${jwtString.inner}` } });
    return response.data;
  },
  vrchat_resolve_2fa: async function (creds_with_tfa: VRChatCredentialsWithTwoFactorAuth): Promise<VRChatCredentialsWithCookie> {
    let jwtString: JwtString = JSON.parse(localStorage.getItem("jwt")!);
    let response = await http.post<VRChatCredentialsWithCookie>('/api/user/platform/vrchat/auth_2fa/resolve', creds_with_tfa, { headers: { Authorization: `Bearer ${jwtString.inner}` } });
    return response.data;
  }
}

