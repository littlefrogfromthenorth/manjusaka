<template>
  <PageWrapper dense contentFullHeight contentBackground fixedHeight>
    <div id="vnc-container" ref="vncContainerRef">
      <div id="btn-container">
        <a-button-group size="small">
          <a-button
            :type="viewOnly ? 'default' : 'primary'"
            @click="setViewMode(false)"
            :disabled="!vncContainerRef"
          >控制模式</a-button>
          <a-button
            :type="viewOnly ? 'primary' : 'default'"
            @click="setViewMode(true)"
            :disabled="!vncContainerRef"
          >查看模式</a-button>
        </a-button-group>
        <a-button
          size="small"
          type="primary"
          @click="toggleFullscreen"
        >{{ isFullscreen ? "退出全屏" : "点击全屏" }}</a-button>
      </div>
    </div>
  </PageWrapper>
</template>

<script lang="ts">
import {
  defineComponent,
  onMounted,
  onBeforeUnmount,
  ref,
  computed,
  nextTick,
} from "vue";
import { PageWrapper } from "/@/components/Page";
import { useGlobSetting } from "/@/hooks/setting";
import { useRouter } from "vue-router";
import { useTabs } from "/@/hooks/web/useTabs";
import { PostApi } from "/@/api";

import RFB from "@novnc/novnc/core/rfb.js";

export default defineComponent({
  name: "AgentVnc",
  components: { PageWrapper },
  setup() {
    const { currentRoute } = useRouter();
    const query = computed(() => currentRoute.value.params);
    const { closeCurrent, setTitle } = useTabs();

    const vncContainerRef = ref<HTMLDivElement | null>(null);
    const cursorRef = ref<HTMLDivElement | null>(null);
    const viewOnly = ref<boolean>(true);
    const isFullscreen = ref<boolean>(false);

    let rfb: any = null; // RFB 实例
    let agentInfo: any = {};

    // 获取代理信息并设置标题
    async function fetchAgentInfo() {
      try {
        const res = await PostApi({
          action: "agentinfo",
          data: { id: query.value.id },
        });
        agentInfo = res;
        setTitle(`VNC - ${res.username}@${res.intranet}`);
      } catch (err) {
        //console.error('Failed to fetch agent info', err);
      }
    }

    function initVnc() {
      if (!vncContainerRef.value) return;

      const { apiUrl = "/manjusaka" } = useGlobSetting();
      const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
      const url = `${protocol}//${window.location.host}${apiUrl}/webproxy/vnc/${query.value.id}`;

      // 创建 RFB 实例
      rfb = new RFB(vncContainerRef.value, url, {
        credentials: {}, // 如需密码认证，可传 { password: 'xxx' }
        shared: false,
        repeaterID: "",
        showDotCursor: true,
        scaleViewport: true, // 自动缩放 Canvas 以适应容器
        //resizeSession: false   远程缩放
      });

      // 事件监听
      rfb.addEventListener("connect", () => {
        //rfb.setEncodings([0]);
        rfb.viewOnly = viewOnly.value;
      });

      rfb.addEventListener("disconnect", (e: any) => {
        if (vncContainerRef.value) {
          vncContainerRef.value.innerHTML =
            '<div style="color:red;padding:20px;">连接已断开</div>';
        }
      });

      rfb.addEventListener("credentialsrequired", (e: any) => {
        // 如果需要密码，可弹窗输入
        const password = prompt("请输入 VNC 密码:");
        if (password !== null) {
          rfb.sendCredentials({ password });
        }
      });

      const resizeObserver = new ResizeObserver(() => {
        if (rfb && rfb.viewOnly === false) {
          rfb.scaleViewport = true;
        }
      });
      resizeObserver.observe(vncContainerRef.value);

      (window as any).__vncResizeObserver = resizeObserver;
    }

    const setViewMode = (viewOnlyMode: boolean) => {
      viewOnly.value = viewOnlyMode;
      if (rfb) {
        rfb.viewOnly = viewOnly.value;
      }
    };

    const toggleViewMode = () => {
      setViewMode(!viewOnly.value);
    };

    const toggleFullscreen = async () => {
      if (!vncContainerRef.value) return;
      if (isFullscreen.value) {
        if (document.exitFullscreen) {
          await document.exitFullscreen();
        } else if ((document as any).webkitExitFullscreen) {
          (document as any).webkitExitFullscreen();
        }
      } else {
        const elem = vncContainerRef.value;
        try {
          if (elem.requestFullscreen) {
            await elem.requestFullscreen();
          } else if ((elem as any).webkitRequestFullscreen) {
            await (elem as any).webkitRequestFullscreen();
          } else {
            console.warn("Fullscreen API not supported");
            return;
          }
        } catch (err) {
          console.error("Failed to enter fullscreen", err);
          return;
        }
      }
    };

    const updateFullscreenState = () => {
      isFullscreen.value = !!(
        document.fullscreenElement ||
        (document as any).webkitFullscreenElement ||
        (document as any).mozFullScreenElement ||
        (document as any).msFullscreenElement
      );
    };

    onMounted(async () => {
      await fetchAgentInfo();
      await nextTick();
      initVnc();
      document.addEventListener("fullscreenchange", updateFullscreenState);
      document.addEventListener("webkitfullscreenchange",updateFullscreenState);
      document.addEventListener("mozfullscreenchange", updateFullscreenState);
      document.addEventListener("MSFullscreenChange", updateFullscreenState);
    });

    onBeforeUnmount(() => {
      if (rfb) {
        rfb.disconnect();
        rfb = null;
      }
      const observer = (window as any).__vncResizeObserver;
      if (observer) {
        observer.disconnect();
      }
      document.removeEventListener("fullscreenchange", updateFullscreenState);
      document.removeEventListener("webkitfullscreenchange", updateFullscreenState);
      document.removeEventListener("mozfullscreenchange",updateFullscreenState);
      document.removeEventListener("MSFullscreenChange", updateFullscreenState);
    });

    return {
      vncContainerRef,
      toggleViewMode,
      setViewMode,
      viewOnly,
      toggleFullscreen,
      isFullscreen,
      rfb,
    };
  },
});
</script>

<style lang="less">
#vnc-container {
  width: 100%;
  height: 100%;
  background-color: #000;
  overflow: hidden;
}
#btn-container {
  position: absolute;
  top: 1px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
}
</style>
