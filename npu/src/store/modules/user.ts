import type { UserInfo } from "/#/store";
import type { ErrorMessageMode } from "/#/axios";
import { defineStore } from "pinia";
import { store } from "/@/store";
import { RoleEnum } from "/@/enums/roleEnum";
import { PageEnum } from "/@/enums/pageEnum";
import { ROLES_KEY, TARGET_KEY, TOKEN_KEY, USER_INFO_KEY } from "/@/enums/cacheEnum";
import { getAuthCache, setAuthCache } from "/@/utils/auth";

import { useI18n } from "/@/hooks/web/useI18n";
import { useMessage } from "/@/hooks/web/useMessage";
import { router } from "/@/router";
import { usePermissionStore } from "/@/store/modules/permission";
import { RouteRecordRaw } from "vue-router";
import { PAGE_NOT_FOUND_ROUTE } from "/@/router/routes/basic";
import { isArray } from "/@/utils/is";
import { h } from "vue";

import { GetUserInfoModel, LoginParams } from "/@/api/sys/model/userModel";
import { GetApi, PostApi } from "/@/api/index";

interface UserState {
  userInfo: Nullable<UserInfo>;
  token?: string;
  target?: string;
  roleList: RoleEnum[];
  sessionTimeout?: boolean;
  lastUpdateTime: number;
}

export const useUserStore = defineStore({
  id: "app-user",
  state: (): UserState => ({
    // user info
    userInfo: null,
    // token
    token: undefined,
    target: undefined,
    // roleList
    roleList: [],
    // Whether the login expired
    sessionTimeout: false,
    // Last fetch time
    lastUpdateTime: 0,
  }),
  getters: {
    getUserInfo(): UserInfo {
      return this.userInfo || getAuthCache<UserInfo>(USER_INFO_KEY) || {};
    },
    getToken(): string {
      return this.token || getAuthCache<string>(TOKEN_KEY);
    },
    getTarget(): string {
      return this.target || getAuthCache<string>(TARGET_KEY);
    },
    getRoleList(): RoleEnum[] {
      return this.roleList.length > 0 ? this.roleList : getAuthCache<RoleEnum[]>(ROLES_KEY);
    },
    getSessionTimeout(): boolean {
      return !!this.sessionTimeout;
    },
    getLastUpdateTime(): number {
      return this.lastUpdateTime;
    },
  },
  actions: {
    setToken(info: string | undefined) {
      this.token = info ? info : ""; // for null or undefined value
      setAuthCache(TOKEN_KEY, info);
    },
    setTarget(info: string | undefined) {
      this.target = info ? info : ""; // for null or undefined value
      setAuthCache(TARGET_KEY, info);
    },
    setRoleList(roleList: RoleEnum[]) {
      this.roleList = roleList;
      setAuthCache(ROLES_KEY, roleList);
    },
    setUserInfo(info: UserInfo | null) {
      this.userInfo = info;
      this.lastUpdateTime = new Date().getTime();
      setAuthCache(USER_INFO_KEY, info);
    },
    setSessionTimeout(flag: boolean) {
      this.sessionTimeout = flag;
    },
    resetState() {
      this.userInfo = null;
      this.token = "";
      this.target = "";
      this.roleList = [];
      this.sessionTimeout = false;
    },
    /**
     * @description: login
     */
    async login(
      action,
      params: LoginParams & {
        goHome?: boolean;
        mode?: ErrorMessageMode;
      }
    ): Promise<GetUserInfoModel | null> {
      try {
        const { goHome = true, mode, ...loginParams } = params;
        const userInfo = await PostApi({action:action, data:loginParams}, mode);

        this.setToken(userInfo.token);
        this.setTarget(userInfo.target);
        
        const { roles = [] } = userInfo;
        if (isArray(roles)) {
          const roleList = roles.map((item) => item.value) as RoleEnum[];
          this.setRoleList(roleList);
        } else {
          userInfo.roles = [];
          this.setRoleList([]);
        }
        this.setUserInfo(userInfo);

        const sessionTimeout = this.sessionTimeout;

        if (sessionTimeout) {
          this.setSessionTimeout(false);
        } else {
          const permissionStore = usePermissionStore();
          if (!permissionStore.isDynamicAddedRoute) {
            const routes = await permissionStore.buildRoutesAction();
            routes.forEach((route) => {
              router.addRoute(route as unknown as RouteRecordRaw);
            });
            permissionStore.setDynamicAddedRoute(true);
          }
          goHome && (await router.replace(PageEnum.BASE_HOME));
        }
      } catch (error) {
        return Promise.reject(error);
      }
    },

    async logout(goLogin = false) {
      this.setToken(undefined);
      this.setSessionTimeout(false);
      this.setUserInfo(null);
      goLogin && router.push(PageEnum.BASE_LOGIN);
    },

    /**
     * @description: Confirm before logging out
     */
    confirmLoginOut() {
      const { createConfirm } = useMessage();
      const { t } = useI18n();
      createConfirm({
        iconType: "warning",
        title: () => h("span", t("sys.app.logoutTip")),
        content: () => h("span", t("sys.app.logoutMessage")),
        onOk: async () => {
          await this.logout(true);
        },
      });
    },
  },
});

// Need to be used outside the setup
export function useUserStoreWithOut() {
  return useUserStore(store);
}
