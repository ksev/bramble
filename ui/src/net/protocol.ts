// @generated by protobuf-ts 2.2.2
// @generated from protobuf file "protocol.proto" (package "protocol", syntax proto3)
// tslint:disable
import { ServiceType } from "@protobuf-ts/runtime-rpc";
import type { BinaryWriteOptions } from "@protobuf-ts/runtime";
import type { IBinaryWriter } from "@protobuf-ts/runtime";
import { WireType } from "@protobuf-ts/runtime";
import type { BinaryReadOptions } from "@protobuf-ts/runtime";
import type { IBinaryReader } from "@protobuf-ts/runtime";
import { UnknownFieldHandler } from "@protobuf-ts/runtime";
import type { PartialMessage } from "@protobuf-ts/runtime";
import { reflectionMergePartial } from "@protobuf-ts/runtime";
import { MESSAGE_TYPE } from "@protobuf-ts/runtime";
import { MessageType } from "@protobuf-ts/runtime";
/**
 * @generated from protobuf message protocol.Sensor
 */
export interface Sensor {
    /**
     * @generated from protobuf field: int32 id = 1;
     */
    id: number;
    /**
     * @generated from protobuf field: float value = 2;
     */
    value: number;
}
/**
 * @generated from protobuf message protocol.Zigbee2MQTTConfig
 */
export interface Zigbee2MQTTConfig {
    /**
     * @generated from protobuf field: string url = 1;
     */
    url: string;
    /**
     * @generated from protobuf field: string username = 2;
     */
    username: string;
    /**
     * @generated from protobuf field: string password = 3;
     */
    password: string;
}
/**
 * @generated from protobuf message protocol.ConfigResult
 */
export interface ConfigResult {
    /**
     * @generated from protobuf oneof: result_oneof
     */
    resultOneof: {
        oneofKind: "success";
        /**
         * @generated from protobuf field: bool success = 4;
         */
        success: boolean;
    } | {
        oneofKind: "message";
        /**
         * @generated from protobuf field: string message = 5;
         */
        message: string;
    } | {
        oneofKind: undefined;
    };
}
// @generated message type with reflection information, may provide speed optimized methods
class Sensor$Type extends MessageType<Sensor> {
    constructor() {
        super("protocol.Sensor", [
            { no: 1, name: "id", kind: "scalar", T: 5 /*ScalarType.INT32*/ },
            { no: 2, name: "value", kind: "scalar", T: 2 /*ScalarType.FLOAT*/ }
        ]);
    }
    create(value?: PartialMessage<Sensor>): Sensor {
        const message = { id: 0, value: 0 };
        globalThis.Object.defineProperty(message, MESSAGE_TYPE, { enumerable: false, value: this });
        if (value !== undefined)
            reflectionMergePartial<Sensor>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: Sensor): Sensor {
        let message = target ?? this.create(), end = reader.pos + length;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case /* int32 id */ 1:
                    message.id = reader.int32();
                    break;
                case /* float value */ 2:
                    message.value = reader.float();
                    break;
                default:
                    let u = options.readUnknownField;
                    if (u === "throw")
                        throw new globalThis.Error(`Unknown field ${fieldNo} (wire type ${wireType}) for ${this.typeName}`);
                    let d = reader.skip(wireType);
                    if (u !== false)
                        (u === true ? UnknownFieldHandler.onRead : u)(this.typeName, message, fieldNo, wireType, d);
            }
        }
        return message;
    }
    internalBinaryWrite(message: Sensor, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        /* int32 id = 1; */
        if (message.id !== 0)
            writer.tag(1, WireType.Varint).int32(message.id);
        /* float value = 2; */
        if (message.value !== 0)
            writer.tag(2, WireType.Bit32).float(message.value);
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message protocol.Sensor
 */
export const Sensor = new Sensor$Type();
// @generated message type with reflection information, may provide speed optimized methods
class Zigbee2MQTTConfig$Type extends MessageType<Zigbee2MQTTConfig> {
    constructor() {
        super("protocol.Zigbee2MQTTConfig", [
            { no: 1, name: "url", kind: "scalar", T: 9 /*ScalarType.STRING*/ },
            { no: 2, name: "username", kind: "scalar", T: 9 /*ScalarType.STRING*/ },
            { no: 3, name: "password", kind: "scalar", T: 9 /*ScalarType.STRING*/ }
        ]);
    }
    create(value?: PartialMessage<Zigbee2MQTTConfig>): Zigbee2MQTTConfig {
        const message = { url: "", username: "", password: "" };
        globalThis.Object.defineProperty(message, MESSAGE_TYPE, { enumerable: false, value: this });
        if (value !== undefined)
            reflectionMergePartial<Zigbee2MQTTConfig>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: Zigbee2MQTTConfig): Zigbee2MQTTConfig {
        let message = target ?? this.create(), end = reader.pos + length;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case /* string url */ 1:
                    message.url = reader.string();
                    break;
                case /* string username */ 2:
                    message.username = reader.string();
                    break;
                case /* string password */ 3:
                    message.password = reader.string();
                    break;
                default:
                    let u = options.readUnknownField;
                    if (u === "throw")
                        throw new globalThis.Error(`Unknown field ${fieldNo} (wire type ${wireType}) for ${this.typeName}`);
                    let d = reader.skip(wireType);
                    if (u !== false)
                        (u === true ? UnknownFieldHandler.onRead : u)(this.typeName, message, fieldNo, wireType, d);
            }
        }
        return message;
    }
    internalBinaryWrite(message: Zigbee2MQTTConfig, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        /* string url = 1; */
        if (message.url !== "")
            writer.tag(1, WireType.LengthDelimited).string(message.url);
        /* string username = 2; */
        if (message.username !== "")
            writer.tag(2, WireType.LengthDelimited).string(message.username);
        /* string password = 3; */
        if (message.password !== "")
            writer.tag(3, WireType.LengthDelimited).string(message.password);
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message protocol.Zigbee2MQTTConfig
 */
export const Zigbee2MQTTConfig = new Zigbee2MQTTConfig$Type();
// @generated message type with reflection information, may provide speed optimized methods
class ConfigResult$Type extends MessageType<ConfigResult> {
    constructor() {
        super("protocol.ConfigResult", [
            { no: 4, name: "success", kind: "scalar", oneof: "resultOneof", T: 8 /*ScalarType.BOOL*/ },
            { no: 5, name: "message", kind: "scalar", oneof: "resultOneof", T: 9 /*ScalarType.STRING*/ }
        ]);
    }
    create(value?: PartialMessage<ConfigResult>): ConfigResult {
        const message = { resultOneof: { oneofKind: undefined } };
        globalThis.Object.defineProperty(message, MESSAGE_TYPE, { enumerable: false, value: this });
        if (value !== undefined)
            reflectionMergePartial<ConfigResult>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: ConfigResult): ConfigResult {
        let message = target ?? this.create(), end = reader.pos + length;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case /* bool success */ 4:
                    message.resultOneof = {
                        oneofKind: "success",
                        success: reader.bool()
                    };
                    break;
                case /* string message */ 5:
                    message.resultOneof = {
                        oneofKind: "message",
                        message: reader.string()
                    };
                    break;
                default:
                    let u = options.readUnknownField;
                    if (u === "throw")
                        throw new globalThis.Error(`Unknown field ${fieldNo} (wire type ${wireType}) for ${this.typeName}`);
                    let d = reader.skip(wireType);
                    if (u !== false)
                        (u === true ? UnknownFieldHandler.onRead : u)(this.typeName, message, fieldNo, wireType, d);
            }
        }
        return message;
    }
    internalBinaryWrite(message: ConfigResult, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        /* bool success = 4; */
        if (message.resultOneof.oneofKind === "success")
            writer.tag(4, WireType.Varint).bool(message.resultOneof.success);
        /* string message = 5; */
        if (message.resultOneof.oneofKind === "message")
            writer.tag(5, WireType.LengthDelimited).string(message.resultOneof.message);
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message protocol.ConfigResult
 */
export const ConfigResult = new ConfigResult$Type();
/**
 * @generated ServiceType for protobuf service protocol.ConfigService
 */
export const ConfigService = new ServiceType("protocol.ConfigService", [
    { name: "Zigbee2MQTT", options: {}, I: Zigbee2MQTTConfig, O: ConfigResult },
    { name: "Listen", serverStreaming: true, options: {}, I: Sensor, O: Sensor }
]);
