import { describe, expect, it } from 'vitest'

import { URI_VLESS } from './vless'

describe('VLESS WebSocket transport', () => {
  const uri =
    'vless://00000000-0000-4000-8000-000000000002@ws.example.test:443?type=ws&security=tls&fp=firefox&sni=sni.example.test&host=host.example.test&path=%2Fsocket%3Fed%3D2048#Example'

  it('preserves ordinary WebSocket TLS without HTTP-upgrade flags', () => {
    const proxy = URI_VLESS(uri)

    expect(proxy.network).toBe('ws')
    expect(proxy.tls).toBeTruthy()
    expect(proxy.servername).toBe('sni.example.test')
    expect(proxy['client-fingerprint']).toBe('firefox')
    expect(proxy['ws-opts']).toEqual({
      headers: { Host: 'host.example.test' },
      path: '/socket?ed=2048',
    })
  })

  it('retains explicit HTTP-upgrade transport behavior', () => {
    const proxy = URI_VLESS(uri.replace('type=ws', 'type=httpupgrade'))

    expect(proxy.network).toBe('ws')
    expect(proxy['ws-opts']?.['v2ray-http-upgrade']).toBe(true)
    expect(proxy['ws-opts']?.['v2ray-http-upgrade-fast-open']).toBe(true)
  })
})
