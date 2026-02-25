import { VariableMap } from "./type";

export class VariablesManager {

  private static _instance: VariablesManager | null = null;
  private constructor() {}

  static get instance(): VariablesManager {
    if (!VariablesManager._instance) {
      VariablesManager._instance = new VariablesManager();
    }
    return VariablesManager._instance;
  }
  
  private _variableMap: VariableMap = {};

  getVariableByKey (key: string) {
    return this._variableMap[key];
  }
}