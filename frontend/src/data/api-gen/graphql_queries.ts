import { gql } from "graphql-request";

export const GetAllDevices = gql`
	query getAllDevices {
		device {
			id,
			name,
			parent,
			features {
				id,
				name,
				direction,
				kind,
				meta,
				value {
					... on Ok {
						value
					},
					... on Err {
						message
					}
				}
			}
		}
	}
`;

export const GetDevice = gql`
	query getDevice($id: String!){
		device(id: $id) {
			id,
			name,
			parent,
			features {
				id,
				name,
				direction,
				kind,
				meta
			}
		}
	}
`;

export const CreateGenericDevice = gql`
	mutation createGenericDevice($name: String!) {
		genericDevice(name: $name)
	}
`;

export const CreateValueBuffer = gql`
	mutation createValueBuffer($deviceId: String!, $name: String!, $kind: ValueKind!, $meta: JSONObject) {
		valueBuffer(deviceId: $deviceId, name: $name, kind: $kind, meta: $meta)
	}
`;

export const DeviceUpdates = gql`
	subscription deviceUpdates {
		device {
			id,
			name,
			parent,
			features {
				id,
				name,
				direction,
				kind,
				meta
			}
		}
	}
`;

export const ValueUpdates = gql`
	subscription valueUpdates {
		values {
			device,
			feature,
			value {
				... on Ok {
					value
				},
				... on Err {
					message
				}
			}
		}
	}
`;

export const AddZigbee2MqttIntegration = gql`
	mutation addZigbee2MqttIntegration($host: String!, $port: Int, $username: String, $password: String) {
		zigbee2Mqtt(host: $host, port: $port, username: $username, password: $password) {
			id
		}
	}
`;