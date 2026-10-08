export const MIHOMO_VERSION = 'v1.19.32'

export const WINDOWS_X64_CORE = {
  archiveSha256:
    '1ee9e6a30c4ba3ba4eb6f0bc7027fe77016a4471eb851a03d44841e3313d4012',
  sha256: '3c107d54ef1b895e4980b53f517d52f3c1137101fa2637347c9ce343fc50d7b2',
}

export const WINDOWS_X64_SERVICE = {
  version: 'v2.7.6',
  archiveSha256:
    '187683b89e777cd150aa2d600684a15cbae665e0a6804399366837d5c73ac06f',
}

const GEO_DATA_URL =
  'https://raw.githubusercontent.com/MetaCubeX/meta-rules-dat/426f62e175218c1f1d0c909188fbff1746f88042'

export const RESOURCES = {
  'Country.mmdb': {
    downloadURL: `${GEO_DATA_URL}/country.mmdb`,
    sha256: 'b13f10cdb414b8db78a64c432d6cd511668415760ee0fb2d17ac9f42de5aba08',
  },
  'ASN.mmdb': {
    downloadURL: `${GEO_DATA_URL}/GeoLite2-ASN.mmdb`,
    sha256: 'e16e2db7ed72adc4443e16d7e33a25120ca58bd8d0f2e902b49ac6c50c3f4815',
  },
  'geosite.dat': {
    downloadURL: `${GEO_DATA_URL}/geosite.dat`,
    sha256: 'dac4b8b3b5bce1d8e59f75f5f2cb3f61e336aa5f4ac7ce465f9dee5e5b0e0457',
  },
  'geoip.dat': {
    downloadURL: `${GEO_DATA_URL}/geoip.dat`,
    sha256: '391b522361c52804e486a98b53d97f3d9c1d3e4e4217bb34954774a0b456b9fd',
  },
  'enableLoopback.exe': {
    downloadURL:
      'https://api.github.com/repos/Kuingsmile/uwp-tool/releases/assets/274024421',
    sha256: 'f0c328376bd5ae0ef3b0eb19ac1f343c24a9abfe7bc1008d3b2f6ef3573d04d1',
  },
}

export const NSIS_PLUGIN = {
  downloadURL:
    'https://nsis.sourceforge.io/mediawiki/images/e/ef/NSIS_Simple_Service_Plugin_Unicode_1.30.zip',
  archiveSha256:
    '58a58f257fc5906e3a0215f67ae7a72f468a8d7d7d3019065423eb704fbfaf66',
  sha256: '1620cdf739f459d1d83411f93648f29dcf947a910cc761e85ac79a69639d127d',
}
