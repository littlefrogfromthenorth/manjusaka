<template>
  <div>
    <BasicTable @register="registerTable">
      <template #action="{ record }">
        <TableAction
          :actions="[
            {
              icon: 'fa-solid:cloud-download-alt',
              label: '下载',
              onClick: handleDownload.bind(null, record),
            },
            {
              icon: 'ant-design:delete-outlined',
              color: 'error',
              label: '删除',
              popConfirm: {
                title: '是否确认删除',
                confirm: handleDelete.bind(null, record),
              },
            },
          ]"
        />
      </template>
    </BasicTable>
  </div>
</template>
<script lang="ts">
import { defineComponent } from "vue";
import { BasicTable, useTable, TableAction } from "/@/components/Table";
import { formatToDate, formatToDateTime } from "/@/utils/dateUtil";
import { GetApi,PostApi } from "/@/api";
import { useModal } from "/@/components/Modal";
import { useMessage } from "/@/hooks/web/useMessage";
import { downloadByUrl } from "/@/utils/file/download";
import { useGlobSetting } from "/@/hooks/setting";
import { BasicUpload } from "/@/components/Upload";

export default defineComponent({
  name: "RoleManagement",
  components: { 
    BasicTable, 
    TableAction,
    BasicUpload
  },
  setup() {
    const { apiUrl = "/manjusaka" } = useGlobSetting();
    const { createMessage } = useMessage();
    const [registerModal, { openModal }] = useModal();
    const [registerTable, { reload }] = useTable({
      title: "NPU上传文件",
      columns: [
        {
          title: "路径",
          dataIndex: "filepath",
          width: 300,
        },
        {
          title: "名称",
          dataIndex: "filename",
        },
        {
          title: "大小",
          dataIndex: "filesize",
          width: 150,
          customRender: ({ record }) => {
            return bytesToSize(record.filesize);
          },
        },
        {
          title: "时间",
          dataIndex: "updateat",
          width: 150,
          customRender: ({ record }) => {
            return formatToDateTime(new Date(record.updateat * 1000));
          },
        },
      ],
      showTableSetting: true,
      bordered: true,
      showIndexColumn: false,
      size: "small",
      clickToRowSelect: false,
      actionColumn: {
        title: "操作",
        dataIndex: "action",
        width: 250,
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
        return {action: "ufilelist", data:{page: page, size: pageSize}};
      },
    });

    const bytesToSize = (bytes: number) => {
      const units = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
      if (bytes == 0) return '-';
      const i = Math.floor(Math.log(bytes) / Math.log(1024));
      const size = (bytes / 1024 ** i).toFixed(0);
      if (i === 0) {
        return '1 KB';
      }
      return `${size} ${units[i]}`;
    };

    function handleDelete(record: Recordable) {
      PostApi({action: "filedelete", data:{ id: record.id }})
        .then(() => {
          createMessage.success("删除成功");
        })
        .catch((e) => {
          //createMessage.error(`删除失败 ${e}`);
        })
        .finally(() => {
          reload();
        });
    }
    
    function handleDownload(record: Recordable) {
      downloadByUrl({url: apiUrl + "/filedownload" + "?id=" + record.id, fileName: record.fileanme});
    }

    return {
      registerTable,
      registerModal,
      handleDelete,
      handleDownload,
    };
  },
});
</script>
