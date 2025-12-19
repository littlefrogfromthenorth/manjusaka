<template>
  <div>
    <BasicTable @register="registerTablefileret">
      <template #action="{ record }">
      <div style="display: flex; justify-content: space-between; align-items: center;">
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
                title: '确认删除？',
                confirm: handleDelete.bind(null, record),
              },
            },
            {
              icon: 'bi:eye',
              label: '预览',
              auth: /\.(PNG|JPG|JPEG|GIF|BMP)$/i.test(record.filename) ? false : true,
              onClick: handleImageView.bind(null, record),
            },
          ]"
        />
        </div>
      </template>
    </BasicTable>
  </div>
</template>
<script lang="ts">
import { defineComponent, reactive } from "vue";

import { BasicTable, useTable, TableAction } from "/@/components/Table";
import { formatToDate, formatToDateTime } from "/@/utils/dateUtil";
import { createImgPreview } from "/@/components/Preview/index";
import { GetApi,PostApi } from "/@/api";
import { useModal } from "/@/components/Modal";
import { useMessage } from "/@/hooks/web/useMessage";
import { downloadByUrl } from "/@/utils/file/download";
import { useGlobSetting } from "/@/hooks/setting";

export default defineComponent({
  name: "RoleManagement",
  components: { BasicTable, TableAction },
  setup() {
    const state = reactive({
      imgList: [],
    });
    const { apiUrl = "/manjusaka" } = useGlobSetting();
    const { createMessage } = useMessage();
    const [registerModal, { openModal }] = useModal();
    const [registerTablefileret, { reload }] = useTable({
      title: "NPC上传文件",
      columns: [
        {
          title: "NPC",
          dataIndex: "agent",
          width: 300,
        },
        {
          title: "路径",
          dataIndex: "filepath",
          align: "right",
        },
        {
          title: "名称",
          dataIndex: "filename",
          align: "left",
        },
        {
          title: "大小",
          dataIndex: "filesize",
          width: 80,
          customRender: ({ record }) => {
            return bytesToSize(record.filesize);
          },
        },
        {
          title: "时间",
          dataIndex: "updateat",
          width: 100,
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
        title: "操作",
        dataIndex: "action",
        width: 220,
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
        return { action:"filelist", data:{page: page, size: pageSize }};
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
        .catch(() => {
          //createMessage.error("删除失败");
        })
        .finally(() => {
          reload();
        });
    }
    function handleDownload(record: Recordable) {
      downloadByUrl({url: apiUrl + "/filedownload" + "?id=" + record.id, fileName: record.fileanme});
    }
    function handleImageView(record: Recordable) {
      let url = apiUrl +"/filedownload?id=" + record.id;
      if (state.imgList.indexOf(url) == -1) {
        state.imgList.push(url);
      }
      createImgPreview({
        imageList: state.imgList,
        defaultWidth: 800,
        scaleStep: 5,
        index: state.imgList.indexOf(url),
      });
    }

    return {
      registerTablefileret,
      registerModal,
      bytesToSize,
      handleDelete,
      handleDownload,
      handleImageView
    };
  },
});
</script>
