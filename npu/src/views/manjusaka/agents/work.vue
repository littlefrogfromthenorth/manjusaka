<template>
    <PageWrapper dense contentFullHeight contentBackground fixedHeight>
    <Tabs size="small">
      <TabPane tab="NPC详情" key="1" :closable="true">
        <AgentInfo :agent="state.agent" :event="state.event" @onsend="OnHttpSend" />
      </TabPane>
      <template #tabBarExtraContent>
        <PopConfirmButton
          title="确定断开连接?"
          type="danger"
          @confirm="CloseWork"
        >断开连接</PopConfirmButton>
      </template>
    </Tabs>
  </PageWrapper>
</template>

<script lang="ts">
import {
  defineComponent,
  ref,
  unref,
  reactive,
  watchEffect,
  onMounted,
  computed,
  toRefs,
} from "vue";

import { PageWrapper } from "/@/components/Page";
import { PopConfirmButton } from "/@/components/Button";
import { useTabs } from "/@/hooks/web/useTabs";
import { useMessage } from "/@/hooks/web/useMessage";
import { useGlobSetting } from "/@/hooks/setting";
import { useWebSocket } from "@vueuse/core";
import { Tabs } from "ant-design-vue";
import { useRouter } from 'vue-router';
import { GetApi,PostApi } from "/@/api";
import { buildUUID } from "/@/utils/uuid";
import { nps } from './nps.js';
import AgentInfo from "./works/info.vue";
export default defineComponent({
  name: "AgentWork",
  components: {
    PageWrapper,
    PopConfirmButton,
    Tabs,
    TabPane: Tabs.TabPane,
    AgentInfo,
  },
  setup() {
    const { currentRoute } = useRouter();
    const query = computed(() => unref(currentRoute).params);
    const { createMessage } = useMessage();
    const { closeCurrent, setTitle } = useTabs();
    const state = reactive({
      uid: buildUUID(),
      agent: {},
      event: {},
    });
    const { apiUrl = "/manjusaka" } = useGlobSetting();
    const server = (window.location.protocol === 'https:' ?"wss:":"ws:") + "//" + window.location.host + apiUrl + "/webproxy/log/" + state.uid;
    const { data, send, close } = useWebSocket(server, {
      autoReconnect: false,
      heartbeat: false,
    });

    function CloseWork() {
      close();
      closeCurrent();
    }

    function OnWsSend(event) {
      if (event && state.agent.id) {
        event.id = state.uid;
        event.sendto = state.agent.id;
        let errmsg = nps.AgentEvent.verify(event);
        if (errmsg) {throw Error(errmsg)};
        var uint8 = nps.AgentEvent.encode(event).finish();
        send(uint8);
      }
    }

    function OnHttpSend(event) {
      if (event && state.agent.id) {
        event.id = state.uid;
        event.sendto = state.agent.id;
        let errmsg = nps.AgentEvent.verify(event);
        if (errmsg) {throw Error(errmsg)};
        var uint8 = nps.AgentEvent.encode(event).finish();
        let binary = '';
        for (let i = 0; i < uint8.length; i++) {
          binary += String.fromCharCode(uint8[i]);
        }
        PostApi({action: "addevent",data:{id:state.agent.id, data: btoa(binary)}});
      }
    }

    function OnInit() {
      PostApi({
        action: "agentinfo",
        data: { id: unref(query).id },
      }).then((e) => {
          state.agent = e;
          setTitle("工作台 - " + state.agent.username + "@" + state.agent.intranet);
      }).catch(() => {
          //createMessage.error("初始化失败");
      });
    }

    watchEffect(() => {
      if (data.value) {
        try {
          data.value
            .arrayBuffer()
            .then((buf) => new Uint8Array(buf))
            .then((bytes) => {
              let event = nps.AgentEvent.decode(bytes);
              state.event = event;
            });
        }catch(e){
          //console.error(e);
        }
      }
    });

    onMounted(() => {
      OnInit()
    });

    return {
      state,
      CloseWork,
      OnHttpSend,
    };
  },
});
</script>
