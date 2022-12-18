export type Result<T> = { Ok: T } | { Err: string };

export interface Value { 
  event: 'value'
  device: string,
  property: string,
  value: Result<number | string | null | boolean> 
}

export type ValueKind = 
  { type: "bool" } |
  { type: "number", unit?: string } |
  { type: "state", possible: string[] } |
  { type: "string" } |
  { type: "any" };

export interface ValueSpec {
  id: string,
  name: string,
  direction: "source" | "sink" | "sourceSink",
  kind: ValueKind,
  meta: Object,
}

export interface Device {
  event: 'device'
  id: string
  name: string
  features: ValueSpec[]
}
