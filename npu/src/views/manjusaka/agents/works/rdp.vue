<template>
  <PageWrapper dense contentFullHeight contentBackground fixedHeight>
    <div id="rdp-container" ref="rdpContainerRef">
      <div id="btn-container">
        <a-button size="small" type="primary" @click="toggleFullscreen">{{
          isFullscreen ? "退出全屏" : "点击全屏"
        }}</a-button>
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

import init, * as IronRdp from "../ironrdpweb/ironrdp_web.js";

export default defineComponent({
  name: "AgentRdp",
  components: { PageWrapper },
  setup() {
    const { currentRoute } = useRouter();
    const query = computed(() => currentRoute.value.params);
    const { closeCurrent, setTitle } = useTabs();

    const rdpContainerRef = ref<HTMLDivElement | null>(null);
    const canvasRef = ref<HTMLCanvasElement | null>(null);
    const viewOnly = ref<boolean>(true);
    const isFullscreen = ref<boolean>(false);
    const session = ref<any>(null);

    let agentInfo: any = {};

    // 获取代理信息并设置标题
    async function fetchAgentInfo() {
      try {
        const res = await PostApi({
          action: "agentinfo",
          data: { id: query.value.id },
        });
        agentInfo = res;
        setTitle(`RDP - ${res.username}@${res.intranet}`);
      } catch (err) {
        console.error("Failed to fetch agent info", err);
      }
    }

    async function startRdp() {
      if (!rdpContainerRef.value) {
        return;
      }

      const { apiUrl = "/manjusaka" } = useGlobSetting();
      const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
      const url = `${protocol}//${window.location.host}${apiUrl}/webproxy/rdp/${query.value.id}`;
      const builder = new IronRdp.SessionBuilder()
	        .username("\\")
	        .password("")
	        .authToken("")
	        .renderCanvas(rdpContainerRef.value)
	        .proxyAddress(url); 

	  const newSession = await builder.connect();
	    session.value = newSession;
        newSession
          .run()
          .then((terminationInfo) => {
            session.value = null;
        }).catch((error) => {
            session.value = null;
        });
    }

    const setViewMode = (viewOnlyMode: boolean) => {
      viewOnly.value = viewOnlyMode;
    };

    const toggleFullscreen = async () => {
      if (!rdpContainerRef.value) return;
      if (isFullscreen.value) {
        if (document.exitFullscreen) {
          await document.exitFullscreen();
        } else if ((document as any).webkitExitFullscreen) {
          (document as any).webkitExitFullscreen();
        } else if ((document as any).mozCancelFullScreen) {
          (document as any).mozCancelFullScreen();
        } else if ((document as any).msExitFullscreen) {
          (document as any).msExitFullscreen();
        }
      } else {
        const elem = rdpContainerRef.value;
        if (elem.requestFullscreen) {
          await elem.requestFullscreen();
        } else if ((elem as any).webkitRequestFullscreen) {
          await (elem as any).webkitRequestFullscreen();
        } else if ((elem as any).mozRequestFullScreen) {
          await (elem as any).mozRequestFullScreen();
        } else if ((elem as any).msRequestFullscreen) {
          await (elem as any).msRequestFullscreen();
        } else {
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

    const handleResize = () => {
      if (session.value && canvasRef.value) {
        const container = rdpContainerRef.value;
        if (container) {
          const width = container.clientWidth;
          const height = container.clientHeight - 40; // 减去按钮区域高度
          canvasRef.value.width = width;
          canvasRef.value.height = height;
          try {
            session.value.resize(width, height);
          } catch (error) {
            console.error("Failed to resize session:", error);
          }
        }
      }
    };

    onMounted(async () => {
      await fetchAgentInfo();
      await nextTick();
      await init();
      await startRdp();

      document.addEventListener("fullscreenchange", updateFullscreenState);
      document.addEventListener("webkitfullscreenchange",updateFullscreenState);
      document.addEventListener("mozfullscreenchange", updateFullscreenState);
      document.addEventListener("MSFullscreenChange", updateFullscreenState);
      window.addEventListener("resize", handleResize);
      document.addEventListener("fullscreenchange", handleResize);
      document.addEventListener("webkitfullscreenchange", handleResize);
      document.addEventListener("mozfullscreenchange", handleResize);
      document.addEventListener("MSFullscreenChange", handleResize);
    });

    onBeforeUnmount(() => {
      if (session.value) {
        try {
          session.value.shutdown();
        } catch (error) {
          console.error("Error shutting down session:", error);
        }
        session.value = null;
      }

      document.removeEventListener("fullscreenchange", updateFullscreenState);
      document.removeEventListener("webkitfullscreenchange",updateFullscreenState);
      document.removeEventListener("mozfullscreenchange",updateFullscreenState);
      document.removeEventListener("MSFullscreenChange", updateFullscreenState);
      document.removeEventListener("fullscreenchange", handleResize);
      document.removeEventListener("webkitfullscreenchange", handleResize);
      document.removeEventListener("mozfullscreenchange", handleResize);
      document.removeEventListener("MSFullscreenChange", handleResize);
      window.removeEventListener("resize", handleResize);

    });

    return {
      rdpContainerRef,
      canvasRef,
      toggleViewMode: () => setViewMode(!viewOnly.value),
      setViewMode,
      viewOnly,
      toggleFullscreen,
      isFullscreen,
      session,
    };
  },
});

</script>

<style lang="less">
#rdp-container {
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
