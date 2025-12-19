<template>
  <div>
    <BasicTable @register="registerTablelisten">
      <template #toolbar>
        <a-button type="primary" @click="handleCreate"> 新建隧道 </a-button>
      </template>
      <template #action="{ record }">
        <TableAction
          :actions="[
            {
              icon: 'ant-design:delete-outlined',
              label: '关停',
              color: 'error',
              popConfirm: {
                title: '确认删除隧道？',
                confirm: handleDelete.bind(null, record),
              },
            },
          ]"
        />
      </template>
    </BasicTable>
    <RoleModal @register="registerModal" @success="handleSuccess" />
  </div>
</template>
<script lang="ts">
import { defineComponent, h, reactive } from "vue";
import { Switch, Radio } from "ant-design-vue";
import { BasicTable, useTable, TableAction } from "/@/components/Table";
import { BasicForm, useForm } from "/@/components/Form/index";
import { Description, DescItem, useDescription } from "/@/components/Description/index";
import { PostApi } from "/@/api";
import { useMessage } from "/@/hooks/web/useMessage";
import { useModal } from "/@/components/Modal";
import { useUserStore } from "/@/store/modules/user";
import RoleModal from "./RoleModal.vue";
import { formatToDate } from "/@/utils/dateUtil";

export default defineComponent({
  name: "ListenManagement",
  components: {
    BasicTable,
    RoleModal,
    TableAction,
    Description,
    BasicForm,
  },
  setup() {
    const state = reactive({
      proxys: [],
    });
    const { createMessage } = useMessage();
    const userStore = useUserStore();
    const [registerModal, { openModal }] = useModal();
    const [registerTablelisten, { reload }] = useTable({
      title: "需要先加载NPC2才能使用隧道代理",
      columns: [
        {
          title: "ID",
          dataIndex: "id",
          width: 300,
        },
        {
          title: "NPC",
          dataIndex: "agent",
        },
        {
          title: "隧道类型",
          dataIndex: "name",
        },
        {
          title: "本地端口",
          dataIndex: "local_port",
        },
        {
          title: "远程地址",
          dataIndex: "remote_addr",
        },
        {
          title: "代理验证",
          dataIndex: "username",
          customRender: ({ record }) => {
            return record.username + '/' + record.password;
          },
        }
      ],
      api: PostApi,
      showTableSetting: true,
      bordered: true,
      showIndexColumn: false,
      actionColumn: {
        width: 100,
        title: "操作",
        dataIndex: "action",
        slots: { customRender: "action" },
        fixed: undefined,
      },
      beforeFetch: function (params) {
        return { action: "proxylist", data:{}};
      },
    });

    function handleCreate() {
      openModal(true, {});
      reload();
    }


    function handleDelete(record: Recordable) {
      PostApi({ action: "proxystop", data: { id: record.id } })
        .then((ret) => {
          createMessage.success("删除成功");
          reload();
        })
        .catch(() => {
          createMessage.error("删除失败");
        });
    }

    function handleSuccess() {
      reload();
    }

    return {
      state,
      registerTablelisten,
      registerModal,
      handleCreate,
      handleDelete,
      handleSuccess,
    };
  },
});
</script>
