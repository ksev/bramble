import type { DocumentNode } from 'graphql';
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

export type Err = {
  __typename?: 'Err';
  message: Scalars['String'];
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
  /** The current value of the feature, ONLY source features will have a value */
  value?: Maybe<Value>;
};

export type Mutation = {
  __typename?: 'Mutation';
  /** Add or change automation for a feature */
  automate: Scalars['Int'];
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
  /** Add Zigbee2Mqtt integration */
  zigbee2Mqtt: Device;
};


export type MutationAutomateArgs = {
  deviceId: Scalars['String'];
  featureId: Scalars['String'];
  program: Scalars['JSON'];
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


export type MutationZigbee2MqttArgs = {
  host: Scalars['String'];
  password?: InputMaybe<Scalars['String']>;
  port?: InputMaybe<Scalars['Int']>;
  username?: InputMaybe<Scalars['String']>;
};

export type Ok = {
  __typename?: 'Ok';
  value: Scalars['JSON'];
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
  values: ValueUpdate;
};

export type Value = Err | Ok;

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

/** A value of a device that has been reported to the system */
export type ValueUpdate = {
  __typename?: 'ValueUpdate';
  /** The id of the device the value is for */
  device: Scalars['String'];
  /** The feature's name on the device the value is for */
  feature: Scalars['String'];
  /** The value of the device, note can be error */
  value: Value;
};

export type GetAllDevicesQueryVariables = Exact<{ [key: string]: never; }>;


export type GetAllDevicesQuery = { __typename?: 'Query', device: Array<{ __typename?: 'Device', id: string, name: string, parent?: string | null, features: Array<{ __typename?: 'Feature', id: string, name: string, direction: ValueDirection, kind: ValueKind, meta: any, value?: { __typename?: 'Err', message: string } | { __typename?: 'Ok', value: any } | null }> }> };

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

export type SetAutomateMutationVariables = Exact<{
  deviceId: Scalars['String'];
  featureId: Scalars['String'];
  program: Scalars['JSON'];
}>;


export type SetAutomateMutation = { __typename?: 'Mutation', automate: number };

export type DeviceUpdatesSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type DeviceUpdatesSubscription = { __typename?: 'Subscription', device: { __typename?: 'Device', id: string, name: string, parent?: string | null, features: Array<{ __typename?: 'Feature', id: string, name: string, direction: ValueDirection, kind: ValueKind, meta: any }> } };

export type ValueUpdatesSubscriptionVariables = Exact<{ [key: string]: never; }>;


export type ValueUpdatesSubscription = { __typename?: 'Subscription', values: { __typename?: 'ValueUpdate', device: string, feature: string, value: { __typename?: 'Err', message: string } | { __typename?: 'Ok', value: any } } };

export type AddZigbee2MqttIntegrationMutationVariables = Exact<{
  host: Scalars['String'];
  port?: InputMaybe<Scalars['Int']>;
  username?: InputMaybe<Scalars['String']>;
  password?: InputMaybe<Scalars['String']>;
}>;


export type AddZigbee2MqttIntegrationMutation = { __typename?: 'Mutation', zigbee2Mqtt: { __typename?: 'Device', id: string } };


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
      value {
        ... on Ok {
          value
        }
        ... on Err {
          message
        }
      }
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
export const SetAutomateDocument = gql`
    mutation setAutomate($deviceId: String!, $featureId: String!, $program: JSON!) {
  automate(deviceId: $deviceId, featureId: $featureId, program: $program)
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
export const ValueUpdatesDocument = gql`
    subscription valueUpdates {
  values {
    device
    feature
    value {
      ... on Ok {
        value
      }
      ... on Err {
        message
      }
    }
  }
}
    `;
export const AddZigbee2MqttIntegrationDocument = gql`
    mutation addZigbee2MqttIntegration($host: String!, $port: Int, $username: String, $password: String) {
  zigbee2Mqtt(host: $host, port: $port, username: $username, password: $password) {
    id
  }
}
    `;
export type Requester<C = {}, E = unknown> = <R, V>(doc: DocumentNode, vars?: V, options?: C) => Promise<R> | AsyncIterable<R>
export function getSdk<C, E>(requester: Requester<C, E>) {
  return {
    getAllDevices(variables?: GetAllDevicesQueryVariables, options?: C): Promise<GetAllDevicesQuery> {
      return requester<GetAllDevicesQuery, GetAllDevicesQueryVariables>(GetAllDevicesDocument, variables, options) as Promise<GetAllDevicesQuery>;
    },
    getDevice(variables: GetDeviceQueryVariables, options?: C): Promise<GetDeviceQuery> {
      return requester<GetDeviceQuery, GetDeviceQueryVariables>(GetDeviceDocument, variables, options) as Promise<GetDeviceQuery>;
    },
    createGenericDevice(variables: CreateGenericDeviceMutationVariables, options?: C): Promise<CreateGenericDeviceMutation> {
      return requester<CreateGenericDeviceMutation, CreateGenericDeviceMutationVariables>(CreateGenericDeviceDocument, variables, options) as Promise<CreateGenericDeviceMutation>;
    },
    createValueBuffer(variables: CreateValueBufferMutationVariables, options?: C): Promise<CreateValueBufferMutation> {
      return requester<CreateValueBufferMutation, CreateValueBufferMutationVariables>(CreateValueBufferDocument, variables, options) as Promise<CreateValueBufferMutation>;
    },
    setAutomate(variables: SetAutomateMutationVariables, options?: C): Promise<SetAutomateMutation> {
      return requester<SetAutomateMutation, SetAutomateMutationVariables>(SetAutomateDocument, variables, options) as Promise<SetAutomateMutation>;
    },
    deviceUpdates(variables?: DeviceUpdatesSubscriptionVariables, options?: C): AsyncIterable<DeviceUpdatesSubscription> {
      return requester<DeviceUpdatesSubscription, DeviceUpdatesSubscriptionVariables>(DeviceUpdatesDocument, variables, options) as AsyncIterable<DeviceUpdatesSubscription>;
    },
    valueUpdates(variables?: ValueUpdatesSubscriptionVariables, options?: C): AsyncIterable<ValueUpdatesSubscription> {
      return requester<ValueUpdatesSubscription, ValueUpdatesSubscriptionVariables>(ValueUpdatesDocument, variables, options) as AsyncIterable<ValueUpdatesSubscription>;
    },
    addZigbee2MqttIntegration(variables: AddZigbee2MqttIntegrationMutationVariables, options?: C): Promise<AddZigbee2MqttIntegrationMutation> {
      return requester<AddZigbee2MqttIntegrationMutation, AddZigbee2MqttIntegrationMutationVariables>(AddZigbee2MqttIntegrationDocument, variables, options) as Promise<AddZigbee2MqttIntegrationMutation>;
    }
  };
}
export type Sdk = ReturnType<typeof getSdk>;