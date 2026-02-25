
export enum VariableDataType {
  BOOLEAN = "BOOLEAN",
  FLOAT = "FLOAT",
  STRING = "STRING",
  ALIAS = "ALIAS",
  COLOR = "COLOR",
  EXPRESSION = "EXPRESSION",
  MAP = "MAP",
  SYMBOL_ID = "SYMBOL_ID",
  FONT_STYLE = "FONT_STYLE",
  TEXT_DATA = "TEXT_DATA",
  INVALID = "INVALID",
  NODE_FIELD_ALIAS = "NODE_FIELD_ALIAS",
}

export enum VariableResolvedDataType {
  BOOLEAN = "BOOLEAN",
  FLOAT = "FLOAT",
  STRING = "STRING",
  COLOR = "COLOR",
  MAP = "MAP",
  SYMBOL_ID = "SYMBOL_ID",
  FONT_STYLE = "FONT_STYLE",
  TEXT_DATA = "TEXT_DATA",
}

export interface ColorRGBA {
  a: number;
  b: number;
  g: number;
  r: number;
}

export interface VariableDataStorageValue {
  boolValue?: boolean;
  textValue?: string;
  floatValue?: number;
  colorValue?: ColorRGBA;
  alias?: string;
  // expressionValue?: Expression;
  // mapValue?: VariableMap;
  // symbolIdValue?: AssetID;
  // fontStyleValue?: VariableFontStyle;
  // textDataValue?: TextData;
  // nodeFieldAliasValue?: NodeFieldAlias;
}

export interface VariableDataStorage {
  value: VariableDataStorageValue;
  dataType:  VariableDataType | string;
  resolvedDataType: VariableResolvedDataType | string;
}


export type VariableModeValueType = Record<string, VariableDataStorage>;

export interface VariableItem {
  type: string;
  name: string;
  variableSetID: string;
  modeValues:VariableModeValueType;
}

export type VariableMap = Record<string, VariableItem>
