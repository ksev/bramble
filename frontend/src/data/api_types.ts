import { GraphQLClient } from 'graphql-request';
import * as Dom from 'graphql-request/dist/types.dom';
import gql from 'graphql-tag';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  JSON: any;
  JSONObject: any;
};

export type Device = {
  __typename?: 'Device';
  /** All the features a device exposes */
  features: Array<Feature>;
  /** Device id, unique device id */
  id: Scalars['String'];
  /** A nicer looking name for the device */
  name: Scalars['String'];
  /** Some devices are spawned by other devices, this tracks that higharchy */
  parent?: Maybe<Scalars['String']>;
};

export type Feature = {
  __typename?: 'Feature';
  /** Which direction does the data flow */
  direction: ValueDirection;
  /** Feature id these are only device unique not global unique */
  id: Scalars['String'];
  /** What type of value this feature has */
  kind: ValueKind;
  /**
   * Json metadata about the feature
   * Common meta data is Number unit a list of possible States for state
   */
  meta: Scalars['JSONObject'];
  /** Feature name, an nice-er to look at name */
  name: Scalars['String'];
};

export type Mutation = {
  __typename?: 'Mutation';
  /**
   * Create a new generic virtual device, this is just a recepticle to
   * attach value buffers to
   */
  genericDevice: Scalars['String'];
  /**
   * Create a value buffer on the target device
   * this device must exist
   */
  valueBuffer: Scalars['String'];
};


export type MutationGenericDeviceArgs = {
  name: Scalars['String'];
};


export type MutationValueBufferArgs = {
  deviceId: Scalars['String'];
  kind: ValueKind;
  meta?: InputMaybe<Scalars['JSONObject']>;
  name: Scalars['String'];
};

export type Query = {
  __typename?: 'Query';
  /** Get all or a specific device */
  device: Array<Device>;
};


export type QueryDeviceArgs = {
  id?: InputMaybe<Scalars['String']>;
};

export type Subscription = {
  __typename?: 'Subscription';
  /** Listen for changes in devices */
  device: Device;
  /**
   * Listen for updates to feature values on devices
   * This will print out all updates on all devices
   */
  values: Value;
};

/** A value of a device that has been reported to the system */
export type Value = {
  __typename?: 'Value';
  /** The id of the device the value is for */
  device: Scalars['String'];
  /** The feature's name on the device the value is for */
  feature: Scalars['String'];
  /** The value of the device, note can be error */
  value: Scalars['JSON'];
};

export enum ValueDirection {
  Sink = 'SINK',
  Source = 'SOURCE',
  SourceSink = 'SOURCE_SINK'
}

export enum ValueKind {
  Bool = 'BOOL',
  Number = 'NUMBER',
  State = 'STATE',
  String = 'STRING'
}

export type GetAllDevicesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAllDevicesQuery = { __typename?: 'Query', device: Array<{ __typename?: 'Device', id: string, name: string, parent?: string | null, features: Array<{ __typename?: 'Feature', id: string, name: string, direction: ValueDirection, kind: ValueKind, meta: any }> }> };

export type GetDeviceQueryVariables = Exact<{
  id: Scalars['String'];
}>;


export type GetDeviceQuery = { __typename?: 'Query', device: Array<{ __typename?: 'Device', id: string, name: string, parent?: string | null, features: Array<{ __typename?: 'Feature', id: string, name: string, direction: ValueDirection, kind: ValueKind, meta: any }> }> };

export type CreateGenericDeviceMutationVariables = Exact<{
  name: Scalars['String'];
}>;


export type CreateGenericDeviceMutation = { __typename?: 'Mutation', genericDevice: string };

export type CreateValueBufferMutationVariables = Exact<{
  deviceId: Scalars['String'];
  name: Scalars['String'];
  kind: ValueKind;
  meta?: InputMaybe<Scalars['JSONObject']>;
}>;


export type CreateValueBufferMutation = { __typename?: 'Mutation', valueBuffer: string };

export type DeviceUpdatesSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type DeviceUpdatesSubscription = { __typename?: 'Subscription', device: { __typename?: 'Device', id: string, name: string, parent?: string | null, features: Array<{ __typename?: 'Feature', id: string, name: string, direction: ValueDirection, kind: ValueKind, meta: any }> } };


export const GetAllDevicesDocument = gql`
    query getAllDevices {
  device {
    id
    name
    parent
    features {
      id
      name
      direction
      kind
      meta
    }
  }
}
    `;
export const GetDeviceDocument = gql`
    query getDevice($id: String!) {
  device(id: $id) {
    id
    name
    parent
    features {
      id
      name
      direction
      kind
      meta
    }
  }
}
    `;
export const CreateGenericDeviceDocument = gql`
    mutation createGenericDevice($name: String!) {
  genericDevice(name: $name)
}
    `;
export const CreateValueBufferDocument = gql`
    mutation createValueBuffer($deviceId: String!, $name: String!, $kind: ValueKind!, $meta: JSONObject) {
  valueBuffer(deviceId: $deviceId, name: $name, kind: $kind, meta: $meta)
}
    `;
export const DeviceUpdatesDocument = gql`
    subscription deviceUpdates {
  device {
    id
    name
    parent
    features {
      id
      name
      direction
      kind
      meta
    }
  }
}
    `;

export type SdkFunctionWrapper = <T>(action: (requestHeaders?:Record<string, string>) => Promise<T>, operationName: string, operationType?: string) => Promise<T>;


const defaultWrapper: SdkFunctionWrapper = (action, _operationName, _operationType) => action();

export function getSdk(client: GraphQLClient, withWrapper: SdkFunctionWrapper = defaultWrapper) {
  return {
    getAllDevices(variables?: GetAllDevicesQueryVariables, requestHeaders?: Dom.RequestInit["headers"]): Promise<GetAllDevicesQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetAllDevicesQuery>(GetAllDevicesDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'getAllDevices', 'query');
    },
    getDevice(variables: GetDeviceQueryVariables, requestHeaders?: Dom.RequestInit["headers"]): Promise<GetDeviceQuery> {
      return withWrapper((wrappedRequestHeaders) => client.request<GetDeviceQuery>(GetDeviceDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'getDevice', 'query');
    },
    createGenericDevice(variables: CreateGenericDeviceMutationVariables, requestHeaders?: Dom.RequestInit["headers"]): Promise<CreateGenericDeviceMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateGenericDeviceMutation>(CreateGenericDeviceDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'createGenericDevice', 'mutation');
    },
    createValueBuffer(variables: CreateValueBufferMutationVariables, requestHeaders?: Dom.RequestInit["headers"]): Promise<CreateValueBufferMutation> {
      return withWrapper((wrappedRequestHeaders) => client.request<CreateValueBufferMutation>(CreateValueBufferDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'createValueBuffer', 'mutation');
    },
    deviceUpdates(variables?: DeviceUpdatesSubscriptionVariables, requestHeaders?: Dom.RequestInit["headers"]): Promise<DeviceUpdatesSubscription> {
      return withWrapper((wrappedRequestHeaders) => client.request<DeviceUpdatesSubscription>(DeviceUpdatesDocument, variables, {...requestHeaders, ...wrappedRequestHeaders}), 'deviceUpdates', 'subscription');
    }
  };
}
export type Sdk = ReturnType<typeof getSdk>;