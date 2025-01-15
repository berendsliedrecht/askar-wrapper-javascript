import { NULL } from '@2060.io/ref-napi'
import { ArcHandle, Jwk, Key, ScanHandle, SessionHandle, StoreHandle } from '@openwallet-foundation/askar-shared'
import { uint8arrayToByteBufferStruct } from './conversion'
import type { ByteBufferStruct, SecretBufferStruct } from './structures'

export type Callback = (err: number) => void
export type CallbackWithResponse = (err: number, response: string) => void

type Argument =
  | Record<string, unknown>
  | ArcHandle
  | StoreHandle
  | SessionHandle
  | ScanHandle
  | unknown[]
  | Date
  | Uint8Array
  | SerializedArgument
  | boolean
  | Jwk
  | Key

type SerializedArgument =
  | string
  | number
  | Callback
  | CallbackWithResponse
  | ArrayBuffer
  | typeof ByteBufferStruct
  | typeof SecretBufferStruct
  | Buffer

type SerializedArguments = Record<string, SerializedArgument>

export type SerializedOptions<Type> = Required<{
  [Property in keyof Type]: Type[Property] extends string
    ? string
    : Type[Property] extends number | boolean | Date | StoreHandle | SessionHandle | ScanHandle
      ? number
      : Type[Property] extends Record<string, unknown> | unknown[]
        ? string
        : Type[Property] extends Buffer | Uint8Array | Key | ArcHandle | Jwk
          ? Buffer
          : Type[Property] extends Callback
            ? Callback
            : Type[Property] extends CallbackWithResponse
              ? CallbackWithResponse
              : Type[Property] extends boolean | undefined
                ? number
                : Type[Property] extends unknown[] | undefined
                  ? string
                  : Type[Property] extends Record<string, unknown> | undefined
                    ? string
                    : Type[Property] extends Date | undefined
                      ? number
                      : Type[Property] extends string | undefined
                        ? string
                        : Type[Property] extends number | undefined
                          ? number
                          : Type[Property] extends Uint8Array | undefined
                            ? Buffer
                            : unknown
}>

const serialize = (arg: Argument): SerializedArgument => {
  switch (typeof arg) {
    case 'undefined':
      return NULL
    case 'boolean':
      return +arg
    case 'string':
      return arg
    case 'number':
      return arg
    case 'function':
      return arg
    case 'object':
      if (arg instanceof Date) {
        return arg.valueOf()
      }
      if (
        arg instanceof ArcHandle ||
        arg instanceof StoreHandle ||
        arg instanceof SessionHandle ||
        arg instanceof ScanHandle
      ) {
        return arg.handle
      }
      if (arg instanceof Key) {
        return arg.handle.handle
      }
      if (arg instanceof Buffer) {
        return uint8arrayToByteBufferStruct(arg) as unknown as Buffer
      }
      if (arg instanceof Uint8Array) {
        return uint8arrayToByteBufferStruct(Buffer.from(arg)) as unknown as Buffer
      }
      if (arg instanceof Jwk) {
        return uint8arrayToByteBufferStruct(Buffer.from(arg.toUint8Array())) as unknown as Buffer
      }
      return JSON.stringify(arg)
    default:
      throw new Error('could not serialize value')
  }
}

const serializeArguments = <T extends Record<string, Argument> = Record<string, Argument>>(
  args: T
): SerializedOptions<T> => {
  const retVal: SerializedArguments = {}
  for (const [key, value] of Object.entries(args)) {
    retVal[key] = serialize(value)
  }
  return retVal as SerializedOptions<T>
}

export { serializeArguments }
