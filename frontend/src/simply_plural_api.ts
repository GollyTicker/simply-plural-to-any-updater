import axios from 'axios'
import parseColor from 'parse-color'

export const http = axios.create({
  baseURL: 'https://api.apparyllis.com',
})

export type PrivacyBucketsResponse = {
  /**
   * bucket uid
   */
  id: string
  content: {
    /**
     * system uid
     */
    uid: string
    name: string
    icon: string
    /**
     * e.g. `#123456`
     */
    color: string
  }
}[]

export type PrivacyBucket = {
  id: string
  name: string
  icon: string
  color: parseColor.Color
}

export async function get_privacy_buckets(token: string): Promise<PrivacyBucket[]> {
  const response = await http.get<PrivacyBucketsResponse>('/v1/privacyBuckets/', {
    headers: { Authorization: `${token}` },
  })

  return response.data.map((bucket) => ({
    id: bucket.id,
    name: bucket.content.name,
    icon: bucket.content.icon,
    color: parseColor(bucket.content.color),
  }))
}
