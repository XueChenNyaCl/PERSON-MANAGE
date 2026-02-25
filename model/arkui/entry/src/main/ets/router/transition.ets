
interface TransitionTranslate {
  x?: number | string;
  y?: number | string;
  z?: number | string;
}

interface TransitionParams {
  duration?: number;
  curve?: Curve | string;
  type?: RouteType;
}

interface TransitionProperty {
  opacity?: number;
  translate?: TransitionTranslate;
}

interface TransitionItem {
  params?: TransitionParams;
  property?: TransitionProperty;
}

type RouteTransitionItem = Record<string, Record<string, TransitionItem>>

type RouteTransition = RouteTransitionItem;

export const routeTransition: RouteTransition = {
  "exit": {},
  "enter": {},
  "backEnter": {},
  "backExit": {}
};

export function getTransitionParams(type: string, animateId: string): PageTransitionOptions {
  if (!animateId) return {};
  const exit = routeTransition[type];
  if (!exit || !exit[animateId]) return {};
  return exit[animateId]?.params || {};
}

export function getTransitionOpacity(type: string, animateId: string): number {
  if (!animateId) return 1;
  const enter = routeTransition[type];
  if (!enter || !enter[animateId]) return 1;

  const opacity = enter[animateId].property?.opacity;

  return opacity === undefined ? 1 : opacity;
}

export function getTransitionTranslate(type: string, animateId: string): TransitionTranslate {
  if (!animateId) return {};
  const enter = routeTransition[type];
  if (!enter || !enter[animateId]) return {};

  return enter[animateId]?.property?.translate || {};
}