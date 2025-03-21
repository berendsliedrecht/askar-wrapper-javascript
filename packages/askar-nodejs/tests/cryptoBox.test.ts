import { deepStrictEqual } from 'node:assert'
import { before, describe, test } from 'node:test'
import { CryptoBox, Key, KeyAlgorithm } from '@openwallet-foundation/askar-shared'
import { setup } from './utils/initialize'

describe('CryptoBox', () => {
  before(setup)

  test('seal', () => {
    const x25519Key = Key.generate(KeyAlgorithm.X25519)

    const message = Uint8Array.from(Buffer.from('foobar'))
    const sealed = CryptoBox.seal({ recipientKey: x25519Key, message })

    const opened = CryptoBox.sealOpen({
      recipientKey: x25519Key,
      ciphertext: sealed,
    })

    deepStrictEqual(opened, message)

    x25519Key.handle.free()
  })
})
