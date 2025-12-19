<template>
  <Form
    class="p-4 enter-x"
    :model="formData"
    :rules="getFormRules"
    ref="formRef"
    v-show="getShow"
    @keypress.enter="handleLogin"
  >
    <FormItem name="account" class="enter-x">
      <Input
        size="large"
        v-model:value="formData.account"
        :placeholder="t('sys.login.userName')"
        class="fix-auto-fill"
      />
    </FormItem>
    <FormItem name="password" class="enter-x">
      <InputPassword
        size="large"
        visibilityToggle
        v-model:value="formData.password"
        :placeholder="t('sys.login.password')"
      />
    </FormItem>
    <FormItem name="captcha" class="enter-x">
      <div class="mt-2 flex flex-grow-0">
        <Input
          size="large"
          visibilityToggle
          v-model:value="formData.captcha"
          placeholder="请输入验证码"
        /><img
          id="captchaimgaddr"
          :src="`../captcha`"
          onclick="this.src='../captcha?t='+Math.random();"
        />
      </div>
    </FormItem>
    <FormItem class="enter-x">
      <Button type="primary" size="large" block @click="handleLogin" :loading="loading">
        {{ t("sys.login.loginButton") }}
      </Button>
    </FormItem>
  </Form>
</template>
<script lang="ts" setup>
import { reactive, ref, unref, computed } from "vue";

import { Form, Input, Button } from "ant-design-vue";

import LoginFormTitle from "./LoginFormTitle.vue";

import { useI18n } from "/@/hooks/web/useI18n";
import { useMessage } from "/@/hooks/web/useMessage";
import { encryptBySha256 } from "/@/utils/cipher";

import { useUserStore } from "/@/store/modules/user";
import { LoginStateEnum, useLoginState, useFormRules, useFormValid } from "./useLogin";
import { useDesign } from "/@/hooks/web/useDesign";

const FormItem = Form.Item;
const InputPassword = Input.Password;
const { t } = useI18n();
const { notification } = useMessage();
const userStore = useUserStore();

const { getLoginState } = useLoginState();
const { getFormRules } = useFormRules();

const formRef = ref();
const loading = ref(false);

const formData = reactive({
  account: "",
  password: "",
  captcha: "",
});

const { validForm } = useFormValid(formRef);

//onKeyStroke('Enter', handleLogin);

const getShow = computed(() => unref(getLoginState) === LoginStateEnum.LOGIN);

async function handleLogin() {
  const data = await validForm();
  if (!data) return;

  try {
    loading.value = true;
    const userInfo = await userStore.login(
      'login',
      {
        captcha: data.captcha,
        username: data.account,
        password: encryptBySha256(data.password),
      });
    if (userInfo) {
      notification.success({
        message: t("sys.login.loginSuccessTitle"),
        description: `${t("sys.login.loginSuccessDesc")}: ${userInfo.username}`,
        duration: 3,
      });
    }
  } catch (error) {
    document.getElementById("captchaimgaddr").src = "../captcha?t=" + Math.random();
  } finally {
    loading.value = false;
  }
}
</script>
