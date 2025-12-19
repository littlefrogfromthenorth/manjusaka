<template>
  <div>
    <BasicTable @register="registerTablepassret">
      <template #toolbar>
        <a-button type="primary" @click="handleCreate"> 新增 </a-button>
      </template>
      <template #action="{ record }">
        <TableAction
          :actions="[
            {
              icon: 'clarity:note-edit-line',
              label: '编辑',
              onClick: handleEdit.bind(null, record),
            },
            {
              icon: 'ant-design:delete-outlined',
              color: 'error',
              label: '删除',
              popConfirm: {
                title: '确认删除?',
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
import { defineComponent } from "vue";

import { BasicTable, useTable, TableAction } from "/@/components/Table";
import { getRoleListByPage } from "/@/api/demo/system";
import { PostApi } from "/@/api";
import { useModal } from "/@/components/Modal";
import { useMessage } from "/@/hooks/web/useMessage";
import { formatToDate } from "/@/utils/dateUtil";

import RoleModal from "./RoleModal.vue";

export default defineComponent({
  name: "RoleManagement",
  components: { BasicTable, RoleModal, TableAction },
  setup() {
    const { createMessage } = useMessage();
    const [registerModal, { openModal }] = useModal();
    const [registerTablepassret, { reload }] = useTable({
      title: "密码收集结果",
      columns: [
        {
          title: "NPC",
          dataIndex: "agent",
        },
        {
          title: "用户名",
          dataIndex: "username",
          align: "right",
        },
        {
          title: "密码",
          dataIndex: "password",
          align: "left",
        },
        {
          title: "来源",
          dataIndex: "passfrom",
        },
        {
          title: "类型",
          dataIndex: "passtype",
          align: "right",
          width: 120,
        },
        {
          title: "时间",
          dataIndex: "updateat",
          width: 120,
          customRender: ({ record }) => {
            return formatToDate(new Date(record.updateat * 1000));
          },
        },
      ],
      showTableSetting: true,
      bordered: true,
      showIndexColumn: false,
      size: "small",
      clickToRowSelect: false,
      actionColumn: {
        width: 150,
        title: "操作",
        dataIndex: "action",
        slots: { customRender: "action" },
        fixed: undefined,
      },
      api: PostApi,
      pagination: {
        pageSizeOptions: [100, 200, 500, 1000],
        pageSize: 100,
      },
      beforeFetch: function (params) {
        const { page, pageSize } = params;
        return { action: "passlist", data:{page: page, size: pageSize }};
      },
    });

    function handleCreate() {
      openModal(true, {
        isUpdate: false,
      });
      reload();
    }

    function handleEdit(record: Recordable) {
      openModal(true, {
        record,
        isUpdate: true,
      });
      reload();
    }

    function handleDelete(record: Recordable) {
      PostApi({ action: "passdelete", data: { id: record.id } })
        .then(() => {
          createMessage.success("删除成功");
        })
        .catch(() => {
          //createMessage.error("删除失败");
        })
        .finally(() => {
          reload();
        });
    }

    function handleSuccess() {
      reload();
    }

    return {
      registerTablepassret,
      registerModal,
      handleCreate,
      handleEdit,
      handleDelete,
      handleSuccess,
    };
  },
});
</script>
