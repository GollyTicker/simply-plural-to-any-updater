import axios from 'axios';
import type { JwtString, UserLoginCredentials, UserUpdatersStatuses } from './sp2any.bindings';

export const http = axios.create({
  baseURL: 'http://localhost:8080',
});

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
  }
}

