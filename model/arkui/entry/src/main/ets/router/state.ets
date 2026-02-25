
class RouterState {
  private _backFrom: string = "";

  public get backFrom() {
    return this._backFrom;
  }

  public setBackFrom(id: string) {
    this._backFrom = id;
  }
}

export const routerState = new RouterState();
