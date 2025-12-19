<template>
  <BasicModal v-bind="$attrs" @register="registerModal" :title="getTitle" @ok="handleSubmit">
    <BasicForm @register="registerForm" />
  </BasicModal>
</template>
<script lang="ts">
import { defineComponent, ref, computed, unref } from "vue";
import { BasicModal, useModalInner } from "/@/components/Modal";
import { BasicForm, useForm } from "/@/components/Form/index";
import { PostApi } from "/@/api";
import { useMessage } from "/@/hooks/web/useMessage";

export default defineComponent({
  name: "RoleModal",
  components: { BasicModal, BasicForm },
  emits: ["success", "register"],
  setup(_, { emit }) {
    const isUpdate = ref(true);
    const rowId = ref("");

    const { createMessage } = useMessage();
    const [registerForm, { setFieldsValue, updateSchema, resetFields, validate }] = useForm({
      labelWidth: 100,
      schemas: [
        {
          field: "id",
          label: "ID",
          component: "Input",
          componentProps: {
            disabled: true,
          },
        },
        {
          field: "username",
          label: "用户名",
          required: true,
          component: "Input",
        },
        {
          field: "password",
          label: "密码",
          required: true,
          component: "Input",
        },
        {
          field: "passfrom",
          label: "密码来源",
          component: "Input",
        },
        {
          field: "passtype",
          label: "类型",
          component: "Input",
        },
      ],
      showActionButtonGroup: false,
      actionColOptions: {
        span: 23,
      },
    });

    const [registerModal, { setModalProps, closeModal }] = useModalInner(async (data) => {
      resetFields();
      setModalProps({ confirmLoading: false });
      isUpdate.value = !!data?.isUpdate;

      if (unref(isUpdate)) {
        rowId.value = data.record.id;
        setFieldsValue({
          ...data.record,
        });
      }
    });

    const getTitle = computed(() => (!unref(isUpdate) ? "新增" : "编辑"));

    async function handleSubmit() {
      try {
        const values = await validate();
        setModalProps({ confirmLoading: true });
        PostApi({ action: "passupdate", data: values })
          .then(() => {
            createMessage.success(`成功!`);
          })
          .catch(() => {
            //createMessage.error("失败 ");
          })
          .finally(() => {});
        closeModal();
        emit("success", { isUpdate: unref(isUpdate), values: { ...values, id: rowId.value } });
      } finally {
        setModalProps({ confirmLoading: false });
      }
    }

    return { registerModal, registerForm, getTitle, handleSubmit };
  },
});
</script>
