import { gql } from "graphql-request";

export const GetAllDevices = gql`
	query getAllDevices {
		device {
			id,
			name,
			parent,
			deviceType,
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
				},
				automate
			},
		}
	}
`;

export const GetDevice = gql`
	query getDevice($id: String!){
		device(id: $id) {
			id,
			name,
			parent,
			deviceType, 
			features {
				id,
				name,
				direction,
				kind,
				meta,
				automate
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
	mutation createValueBuffer($deviceId: String!, $name: String!, $kind: ValueKind!, $meta: JSON) {
		valueBuffer(deviceId: $deviceId, name: $name, kind: $kind, meta: $meta)
	}
`;

export const SetAutomation = gql`
	mutation setAutomate($deviceId: String!, $featureId: String!, $program: JSON!) {
	  automate(deviceId: $deviceId, featureId: $featureId, program: $program)
	}
`

export const DeviceUpdates = gql`
	subscription deviceUpdates {
		device {
			id,
			name,
			parent,
			deviceType,
			features {
				id,
				name,
				direction,
				kind,
				meta,
				automate
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

export const SetSunLocation = gql`
	mutation setSunLocation($lat: Float!, $lon: Float!) {
		sunLocation(lat: $lat, lon: $lon)
	}	
`;