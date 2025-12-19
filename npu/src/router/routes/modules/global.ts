import type { AppRouteModule } from "/@/router/types";
import { LAYOUT } from "/@/router/constant";
const IFrame = () => import('/@/views/sys/iframe/FrameBlank.vue');

const routes: AppRouteModule = {
    path: "/global",
    redirect: "/project",
    component: LAYOUT,
    meta: {
      hideChildrenInMenu: false,
      orderNo: 100,
      title: "项目设置",
    },
    children: [{
          path: "/project",
          name: "GProjects",
          component: () => import("/@/views/manjusaka/projects/index.vue"),
          meta: {title: "目标设定",}
      },{
          path: "/listen",
          name: "GListens",
          component: () => import("/@/views/manjusaka/listens/index.vue"),
          meta: {title: "监听器",}
      },/*{
          path: "/csload",
          name: "Gcsload",
          component: () => import("/@/views/manjusaka/csload/index.vue"),
          meta: {title: "生成器",}
      },*/{
          path: "/setting",
          name: "GSettings",
          component: () => import("/@/views/manjusaka/settings/index.vue"),
          meta: {title: "全局设置",}
      },{
        path: 'https://github.com/YDHCUI/manjusaka',
        name: 'about',
        component: IFrame,
        meta: {title: "GITHUB",}
    }]
  };

export default routes;
