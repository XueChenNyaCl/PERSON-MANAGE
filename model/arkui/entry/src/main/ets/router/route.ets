
import { routerState } from "./state";
import { getTransitionOpacity, getTransitionParams, getTransitionTranslate } from "./transition";

interface routerResult {
  pageShow: Function;
  pageHide: Function;
  pageExit: Function;
  pageExitOpacity: Function;
  pageExitTranslate: Function;
  pageEnter: Function;
  pageEnterOpacity: Function;
  pageEnterTranslate: Function;
  triggerId: string;
  targetId: string;
}

export function handleRouter(pageId: string, uiContext: UIContext): routerResult {
  let routerLen = "";
  let isRouterBack = false;
  let isBack = false;
  let from = '';
  let triggerId = "";
  let targetId = "";
  const getRouterParams = () => {
    const params = uiContext.getRouter().getParams() as Record<string, string>
    if (!params) {
      return;
    }
    from = params.from;
  }
  const diffIsBack = () => {
    const len = uiContext.getRouter().getLength();
    const isBack = Number(len) < Number(routerLen);
    isRouterBack = isBack;
    isBack && routerState.setBackFrom(pageId);
  }
  const getBackFrom = () => {
    isBack = !!routerState.backFrom;
    if (!routerState.backFrom) {
      return;
    }
    routerState.setBackFrom('');
  }
  const pageShow = () => {
    getRouterParams();
    getBackFrom();
    routerLen = uiContext.getRouter().getLength();
  }
  const pageHide = () => {
    diffIsBack();
  }
  const pageExit = () => {
    return getTransitionParams(isRouterBack ? 'backExit' : 'exit',
      isRouterBack ? `${from}_${pageId}` : `${triggerId}_${targetId}`)
  }
  const pageExitOpacity = () => {
    return getTransitionOpacity(isRouterBack ? 'backExit' : 'exit',
      isRouterBack ? `${from}_${pageId}` : `${triggerId}_${targetId}`)
  }
  const pageExitTranslate = () => {
    return getTransitionTranslate(isRouterBack ? 'backExit' : 'exit',
      isRouterBack ? `${from}_${pageId}` : `${triggerId}_${targetId}`)
  }
  const pageEnter = () => {
    return getTransitionParams(isBack ? 'backEnter' : 'enter',
      isBack ? `${triggerId}_${targetId}` : `${from}_${pageId}`)
  }
  const pageEnterOpacity = () => {
    return getTransitionOpacity(isBack ? 'backEnter' : 'enter',
      isBack ? `${triggerId}_${targetId}` : `${from}_${pageId}`)
  }
  const pageEnterTranslate = () => {
    return getTransitionTranslate(isBack ? 'backEnter' : 'enter',
      isBack ? `${triggerId}_${targetId}` : `${from}_${pageId}`)
  }
  return {
    pageShow,
    pageHide,
    pageExit,
    pageExitOpacity,
    pageExitTranslate,
    pageEnter,
    pageEnterOpacity,
    pageEnterTranslate,
    triggerId,
    targetId,
  };
}
