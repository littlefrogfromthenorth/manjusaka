<template>
    <PageWrapper dense contentFullHeight contentBackground fixedHeight>
          <BasicTable @register="registerTableagents">
          <template #action="{ record }">
            <TableAction
              :actions="[{
                  icon: 'bi:eye',
                  label: '连接',
                  onClick: handleAgentWork.bind(null, record),
                },{
                  icon: 'fluent:delete-24-filled',
                  color: 'error',
                  label: '删除',
                  onClick: handleAgentDel.bind(null, record),
                }]"
            />
          </template>
        </BasicTable>
  </PageWrapper>
</template>
<script lang="ts">
import {
  defineComponent,
  ref,
  unref,
  onMounted,
  onUnmounted,
  reactive,
  watchEffect,
  computed,
  toRefs,
  h,
} from "vue";
import { Modal } from "ant-design-vue";
import { BasicDrawer, useDrawerInner } from "/@/components/Drawer";
import { BasicTable, useTable, TableAction } from "/@/components/Table";
import { Time } from "/@/components/Time";
import { Icon } from "/@/components/Icon";
import { createImgPreview } from "/@/components/Preview/index";
import { PageWrapper } from "/@/components/Page";
import { PageEnum } from "/@/enums/pageEnum";
import { formatToDateTime, formatToDate, dateUtil } from "/@/utils/dateUtil";
import { useMessage } from "/@/hooks/web/useMessage";
import { useTabs } from "/@/hooks/web/useTabs";
import { useGo } from "/@/hooks/web/usePage";
import { useRouter } from 'vue-router';
import { PopConfirmButton } from "/@/components/Button";
import { GetApi,PostApi } from "/@/api";
import { useGlobSetting } from "/@/hooks/setting";
import { useWebSocket } from "@vueuse/core";
import { buildUUID } from "/@/utils/uuid";
import { nps } from './nps.js';

export default defineComponent({
  name: "AgentList", // 缓存
  components: {
    BasicTable,
    TableAction,
    Time,
    Modal,
    BasicDrawer,
    Icon,
    PageWrapper,
    PopConfirmButton,
  },
  setup() {
    let timer = null;
    const state = reactive({
      uid: buildUUID(),
      agents: {}
    });
    const getAgents = computed(() => {
      const data: [] = [];
      Object.keys(state.agents).forEach((key) => {
        data.push(state.agents[key]);
      });
      return data;
    });
    const { apiUrl = "/manjusaka" } = useGlobSetting();
    const server = (window.location.protocol === 'https:' ?"wss:":"ws:") + "//" + window.location.host + apiUrl + "/webproxy/log/" + state.uid;
    const { data, send, close } = useWebSocket(server, {
      autoReconnect: false,
      heartbeat: false,
    });
    const go = useGo();
    const { currentRoute,replace } = useRouter();
    const { createMessage } = useMessage();
    const [registerTableagents, { reload }] = useTable({
      columns: [{
        width: 0,
        dataIndex: "id",
      },{
        title: "存活",
        align:'left',
        width: 120,
        dataIndex: "updateat",
        customRender: ({ record }) => {
          return h(Time, {
            step: 1,
            value: formatToDateTime(record.updateat * 1000),
          });
        },
      },{
        title: "系统类型",
        align:'left',
        dataIndex: "platform",
        customRender: ({ record }) => {
          return [h(Icon, {
              size: 20,
              icon: record.platform == 'windows' ? 'devicon:windows8': 'devicon:linux', 
              //icon: record.platform == 'windows' ? 'cib:windows': 'cib:linux', 
              //wpf:macos raphael:android raphael:ios
          }), '   '+ record.platform + "." + record.arch];
        },
      },{
        title: "内网地址",
        align:'left',
        dataIndex: "intranet",
      },{
        title: "用户名",
        align:'left',
        dataIndex: "username",
      },{
        title: "主机名",
        align:'left',
        dataIndex: "hostname",
      },{
        title: "进程名称",
        align:'left',
        dataIndex: "process",
        customRender: ({ record }) => {
          return !record.process ? record.process : record.process.replaceAll("\\", "/").split("/").pop();
        },
      },{
        title: "备注",
        dataIndex: "note",
        edit: true,
      }],
      clickToRowSelect: true,
      bordered: true,
      striped: true,
      showIndexColumn: true,
      pagination: false,
      tableSetting: {
        size: true,
        setting: true,
      },
      size: "small",
      beforeEditSubmit: SetNpcNote,
      dataSource: getAgents,
      actionColumn: {
        title: "操作",
        dataIndex: "action",
        slots: { customRender: "action" },
        fixed: undefined,
      },
    });

    function handleAgentWork(record){
      go("/agentwork/"+record.id);
    }

    function SetNpcNote({ record, index, key, value }) {
      PostApi({action: "agentnote", data: { id: record.id, data: value }});
    }

    function handleAgentDel(record) {
      let id =  record.id ? record.id : "all";
      PostApi({
        action: "agentdel",
        data: { id: id },
      }).then((e) => {
          state.agents = e;
          createMessage.success(`删除成功。 ${id}`);
      }).catch(() => {
          //createMessage.error("删除失败");
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
              if (event.list) {
                event.list.agents.forEach(function (agent) {
                  state.agents[agent.id] = agent;
                  if (event.list.status == "create") {
                    createMessage.info(`您有新的NPC上线:${agent.hostname}\\${agent.username}@${agent.intranet}`);
                  }
                });
              }
            });
        }catch(e){
          //console.error(e);
        }
      }
    });

    onMounted(() => {
      GetApi({ action: "agentget" })
      .then((e) => {
        e.forEach(function (agent) {
          state.agents[agent.id] = agent;
        });
      });
    });

    onUnmounted(() => {});

    return {
      state,
      registerTableagents,
      getAgents,
      SetNpcNote,
      handleAgentWork,
      handleAgentDel,
    };
  },
});
</script>

