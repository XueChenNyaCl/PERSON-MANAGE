
import { display } from '@kit.ArkUI';

interface ScreenAdapter {
  designWidth: number;
  screenWidth: number;
  scale: number;
  toPx: (designPx: number) => number;
  toPy: (designPx: number) => number;
}

const getScreenWidth = (): number => {
  const displayInfo = display.getDefaultDisplaySync();
  return displayInfo.width / displayInfo.densityPixels;
};

export const screenAdapter: ScreenAdapter = {
  designWidth: 750,
  screenWidth: getScreenWidth(),
  get scale() {
    return screenAdapter.screenWidth / screenAdapter.designWidth;
  },
  toPx(designPx: number) {
    return designPx * screenAdapter.scale;
  },
  toPy(designPy: number) {
    return designPy * screenAdapter.scale;
  }
};

export const toPx = screenAdapter.toPx;

export const toPy = screenAdapter.toPx;
