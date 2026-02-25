

import { VariablesManager } from "../variables/VariablesManager";
import { ColorRGBA, VariableDataStorage } from "../variables/type";
import { VariablesSetManager } from "../variablesset/VariablesSetManager";
import { LayerVariableInfoManager } from "./LayerVariableInfoManager";

export type LayerSetToMode = Record<string, string>;

// 变量集到模式的映射
export interface LayerVariableInfoValue {
  guid?: string;
  parentGuids?: string[];
  setToMode?: LayerSetToMode;
  childGuids: string[];
  colorValue?: varValue[];
  booleValue?: varValue[];
  floatValue?: varValue[];
  stringValue?: varValue[];
}
export interface varValue {
  varKey: string;
  varValue: string | number | boolean;
}
export type VariableLayerInfo = Record<string, LayerVariableInfoValue>;

// 获取变量值
export function getVariableValue(variable: VariableDataStorage) {
  switch (variable.dataType) {
    case "STRING":
      return variable.value.textValue;
    case "FLOAT":
      return variable.value.floatValue;
    case "BOOLEAN":
      return variable.value.boolValue;
    case "COLOR": {
      const color = variable.value.colorValue as ColorRGBA;
      return "rgba(" + Math.round(color.r) + ", " + Math.round(color.g) + ", " + Math.round(color.b) + ", " +
        (color.a / 255) + ")";
    }
    default:
      return "";
  }
}

// 所有图层实例的父类
export class LayerVariableManager {
  private static _instance: LayerVariableManager | null = null;
  static get instance(): LayerVariableManager {
    if (!LayerVariableManager._instance) {
      LayerVariableManager._instance = new LayerVariableManager();
    }
    return LayerVariableManager._instance;
  }

  // 获取父级模式ID（迭代方式）
  private getParentSetModeId(guid:string, setId: string): string {
    const currentLayer = LayerVariableInfoManager.instance.variableLayerInfo[guid];
    if (!currentLayer) return "";
    // 要检查的 guid 列表：自己 + parentGuids
    const allGuids = [guid, ...(currentLayer.parentGuids ?? [])];
    for (const item of allGuids) {
      const layer = LayerVariableInfoManager.instance.variableLayerInfo[item];
      const modeId = layer?.setToMode?.[setId];
      if (modeId) {
        return modeId;
      }
    }
    return "";
  }

  private getDefaultModeBySetId(guid: string, setId: string) {
    let curModeId = this.getParentSetModeId(guid, setId);
    if (!curModeId) {
      const variableSetMode =
        VariablesSetManager.instance.getVariableSetByKey(
          setId,
        );
      curModeId = variableSetMode[0]?.id || "";
    }
    return curModeId;
  }

  private getVarOriginData(
     guid: string,
     varKey: string,
     modeId?: string,
   ): VariableDataStorage | null {
     const variable = VariablesManager.instance.getVariableByKey(varKey);
     if (!variable) return null;
     // 确定使用的模式ID
 
     let curModeId =
       modeId || this.getDefaultModeBySetId(guid, variable.variableSetID);
     // 获取模式值
     let modeValue = variable.modeValues[curModeId];
     if (!modeId && !modeValue) return null;
     else if (!modeValue) {
       // 传了modeId但是找不到当前模式值, 则使用默认模式
       curModeId = this.getDefaultModeBySetId(guid, variable.variableSetID);
       modeValue = variable.modeValues[curModeId];
       if (!modeValue) return null;
     }
     // 处理别名
     if (modeValue.dataType === "ALIAS") {
       const aliasKey = modeValue.value.alias;
       if (!aliasKey) return null;
       return this.getVarOriginData(guid, aliasKey, curModeId);
     }
     return modeValue;
   }

  // 类型安全的获取方法
  getColor(guid: string, varKey: string): string {
    const data = this.getVarOriginData(guid, varKey);
    const value = data ? getVariableValue(data) : null;
    return typeof value === "string" ? value : "rgba(0, 0, 0, 1)";
  }

  getString(guid: string, varKey: string): string {
    const data = this.getVarOriginData(guid, varKey);
    const value = data ? getVariableValue(data) : null;
    return typeof value === "string" ? value : typeof value === "number" ? String(value) : "";
  }

  getFloat(guid: string, varKey: string): number {
    const data = this.getVarOriginData(guid, varKey);
    const value = data ? getVariableValue(data) : null;
    return typeof value === "number" ? value : 0;
  }

  getBoolean(guid: string, varKey: string): boolean {
    const data = this.getVarOriginData(guid, varKey);
    const value = data ? getVariableValue(data) : null;
    return typeof value === "boolean" ? value : false;
  }
}

