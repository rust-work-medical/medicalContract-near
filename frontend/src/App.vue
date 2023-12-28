<script setup>
import { ref, onMounted } from 'vue';
import { Wallet } from './near-wallet';
const CONTRACT_ADDRESS ="dev-1702995940967-73575375528011";
const wallet = new Wallet({ createAccessKeyFor: CONTRACT_ADDRESS })
let isSignedIn = ref(false);
const accountId = ref("");
let role = ref("");
import {ElMessage, ElMessageBox} from 'element-plus'
import {Contract} from "near-api-js";
const dialogVisible = ref(false)
const selectedRole = ref('');
//const contract = new Contract({ acccount:"huamen.testnet",contractId: CONTRACT_ADDRESS, walletToUse: wallet });
const signIn = () => {
  wallet.signIn();
  accountId.value = wallet.walletSelector.store.getState().accounts[0].accountId;
  console.log(accountId.value)
  getRole()
};

const signOut = () => {
  wallet.signOut();
  checkAuth();
};

const checkAuth = async () => {
  isSignedIn.value = wallet.walletSelector.isSignedIn();
  if (isSignedIn.value) {
    accountId.value = wallet.walletSelector.store.getState().accounts[0].accountId;
  } else {
    accountId.value = null;
  }
};

// onMounted(() => {
//   checkAuth(); // 页面加载时检查认证状态
// });
const startUp = async () => {
  isSignedIn.value = await wallet.startUp();
  console.log(isSignedIn.value);
  // const urlParams = new URLSearchParams(window.location.search);
  // const txhash = urlParams.get("transactionHashes")
  // console.log("txhash=",txhash)
  // if(txhash !== null){
  //   // Get result from the transaction
  //   let result = await contract.getDonationFromTransaction(txhash)
  //   console.log("result",result)
  // }
};
const setRole = async () => {
  dialogVisible.value=true;
  // let result=await wallet.callMethod({ method: 'get_role', contractId: CONTRACT_ADDRESS });
  // console.log(result)
};
const confirmRole = async () => {
  // 检查是否选择了角色
  if (!selectedRole.value) {
    ElMessage({message:'你没有选择角色',type:'warning'})
    // 可以添加一些提示或逻辑处理
    return;
  }
  console.log(selectedRole.value)
  // 调用 get_role 方法，传递选择的角色
  try{
    let result=await wallet.callMethod({ method: 'register',args: {role: selectedRole.value }, contractId: CONTRACT_ADDRESS});
    ElMessage({message:'设置成功',type:'success'})
    console.log(result)
    role.value=selectedRole.value
  }catch (error)
  {
    ElMessage('发生意外错误')
  }

};
const getRole = async () => {
  if (wallet.isCallback) {
    try {
      // 使用 transaction.result 获取回调返回的结果
      const result = wallet.transaction && wallet.transaction.result;
      console.log('Callback result:', result);
      role.value = result;
    } catch (error) {
      console.error('Callback error:', error);
    }
  } else {
    try {
      const result = await wallet.callMethod({method: 'get_role', args: {}, contractId: CONTRACT_ADDRESS});
      console.log("result=", result.value)
      role.value = result;
      console.log(result);
    } catch (error) {
      ElMessage(error)
    }
  }
};

startUp();

</script>

<template>
<!--  <div>-->
<!--    <h1>NEAR Wallet Login</h1>-->

<!--    <div v-if="isSignedIn">-->
<!--      <p>已登录，账户名: {{ accountId }}</p>-->
<!--      <button @click="signOut">退出</button>-->
<!--    </div>-->

<!--    <div v-else>-->
<!--      <button @click="signIn">登录 NEAR 钱包</button>-->
<!--    </div>-->
<!--  </div>-->
  <el-row class="login-page">
    <el-col  class="bg">
    <div v-if="!isSignedIn" class="form">
      <div class="login-container">
        <p class="login-info">请登录 NEAR 钱包</p>
        <el-button class="login-button" type="primary" @click="signIn">登录</el-button>
      </div>
    </div>
      <div  v-if="isSignedIn" class="form">
        <div class="login-container">
        <p class="login-info"> {{ role ? `请选择你的角色(当前角色为${role})` : '当前没有角色,请选择你的角色' }}</p>
        <el-button class="login-button" type="primary" @click="setRole">设置角色</el-button>
          <el-button class="login-button" type="primary" @click="getRole">查询角色</el-button>
        </div>
      </div>
    </el-col>
  </el-row>


  <el-dialog
      v-model="dialogVisible"
      title="设置角色"
      width="30%"
  >
    <span>当前角色为:{{role}}</span>
    <el-radio-group v-model="selectedRole">
      <el-radio label="doctor">医生</el-radio>
      <el-radio label="patient">病人</el-radio>
      <el-radio label="pharmacy">药房</el-radio>
    </el-radio-group>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmRole">
          确认
        </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<style scoped>
.login-page {
  height: 100vh;
  background-color: #fff;
}
.bg {
  background: url('@/assets/login_bg.jpg') no-repeat center / cover;
  border-radius: 0 20px 20px 0;
}

.form {
  display: flex;
  align-items: center;
  justify-content: center; /* 添加 justify-content 属性 */
  height: 100%; /* 添加 height: 100%; 以确保垂直方向占满整个容器 */
}

.login-container {
  text-align: center;
  flex: 1; /* 添加 flex 属性 */
}

.login-info {
  font-size: 18px;
  margin-bottom: 20px;
  color: #fff; /* 修改字体颜色为白色 */
}

.login-button {
  font-size: 16px;
  color: #fff; /* 修改字体颜色为白色 */
}
</style>