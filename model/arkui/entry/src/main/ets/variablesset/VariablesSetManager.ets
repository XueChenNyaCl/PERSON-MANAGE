export interface VariableSetItem {
  id: string;
  name: string;
}

export type VariableSetMap = Record<string, VariableSetItem[]>;

export class VariablesSetManager {

  private static _instance: VariablesSetManager | null = null;

  private constructor() {}

  static get instance(): VariablesSetManager {
    if (!VariablesSetManager._instance) {
      VariablesSetManager._instance = new VariablesSetManager();
    }
    return VariablesSetManager._instance;
  }
  
  private _variableSetMap: VariableSetMap = {};

   getVariableSetByKey(key: string) {
    return this._variableSetMap[key];
  }
}
