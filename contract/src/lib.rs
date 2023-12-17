use std::collections::HashMap;
use std::ptr::null;
use std::fmt;
// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::log_str;
use near_sdk::serde::{Serialize,Deserialize};
use near_sdk::near_bindgen;
use near_sdk::{Balance,AccountId,env};
use near_sdk::collections::{LookupMap, Vector,UnorderedMap,UnorderedSet};
use near_sdk::test_utils::VMContextBuilder;
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
//医疗记录
pub struct MedicalRecord {
    //医生的账户
  pub doctor: String, 
  //病例详情
  pub detail: String,
  //看病时间
  pub time: String
}


impl fmt::Display for MedicalRecord {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        write!(f, "MedicalRecord(doctor: {}, detail: {}, time: {})", self.doctor, self.detail, self.time)
    }
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    //账户身份
    identify:UnorderedMap<String, String>,
    //病例详情
    pub patient_record: UnorderedMap<String, Vector<MedicalRecord>>,
    //授权病例查询列表
    pub allow_record: UnorderedMap<String, UnorderedSet<String>>,
}
const MIN_STORAGE: Balance = 1_100_000_000_000_000_000_000_000; //1.1Ⓝ
const POINT_ONE: Balance = 100_000_000_000_000_000_000_000;//0.1N
// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self 
        {   
            //需要注意，每一个区块链接口提供的数据结构在初始化的时候都需要添加一个前缀，如果是嵌套结构，在嵌套结构里也需要添加前缀，前缀可以使用账户ID或者其他形式，前缀不能一样
            patient_record: UnorderedMap::new(b"p"),
            identify: UnorderedMap::new(b"i"),
            allow_record:UnorderedMap::new(b"a")
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    //注册
    pub fn register(&mut self,role:String) -> bool {
        self.identify.insert(&env::signer_account_id().as_str().to_string(),&role);
        log_str(&format!("Saving greeting: {role}"));
        log_str(&format!("{}",env::signer_account_id().as_str().to_string()));
        return true;
    }

    //查询用户当前角色
    pub fn get_role(&self) -> String {
        
        let val = match self.identify.get(&env::signer_account_id().as_str().to_string()) {
            Some(x) => return x,
            None => panic!("该账户还没有角色")
          };
    }
    //添加病例
    pub fn add_record(&mut self,patient:String,detail:String)->bool{
        let val = match self.identify.get(&env::signer_account_id().as_str().to_string()) {
            Some(x) => {
                assert!(x=="doctor","该账户不是医生，不能看病");
                match self.identify.get(&patient){
                    Some(patient_id) => {
                        
                        let medical_record = MedicalRecord {
                            doctor: env::signer_account_id().as_str().to_string(),
                            detail: detail,
                            time: env::block_timestamp().to_string(),
                        };
                        //如果存在直接插入一条新记录，如果不存在则新建一个Vector,并插入
                        if(self.patient_record.get(&patient_id).is_some()){
                            self.patient_record.get(&patient_id).unwrap().push(&medical_record);
                        }else {
                            let prefix: Vec<u8> = 
                            [
                                b"m".as_slice(),
                                &near_sdk::env::sha256_array(patient_id.as_bytes()),
                            ].concat();
                            let mut patient_recode_vec:Vector<MedicalRecord>=Vector::new(prefix);
                            patient_recode_vec.push(&medical_record);
                            self.patient_record.insert(&patient_id,&patient_recode_vec);
                        }
                        log_str(&format!("保存病例成功: {medical_record}"));
                        //TODO 这里的时间总是为0,不确定是不是测试环境导致的，后续还需要上链后测试
                        log_str(&format!("当前时间: {}",env::block_timestamp()));
                        return true;
                    },
                    None => return false,
                }

            },
            None => panic!("该账户还没有角色")
          };
    }
    //授权某人查询病例
    pub fn add_allow_record(&mut self,doctor:String)->bool{
        assert!(self.identify.get(&doctor).is_some(), "授权账户没有角色");
        assert!(self.identify.get(&env::signer_account_id().as_str().to_string()).is_some(), "该账户还没有角色");
        //如果允许列表已经含有该角色的授权列表
        if(self.allow_record.get(&env::signer_account_id().as_str().to_string()).is_some()){
            self.allow_record.get(&env::signer_account_id().as_str().to_string()).unwrap().insert(&doctor);
        }else {
            let prefix: Vec<u8> = 
            [
                b"a".as_slice(),
                &near_sdk::env::sha256_array(&env::signer_account_id().as_str().to_string().as_bytes()),
            ].concat();
            let mut patient_recode_vec:UnorderedSet<String>=UnorderedSet::new(prefix);
            patient_recode_vec.insert(&doctor);
            self.allow_record.insert(&env::signer_account_id().as_str().to_string(),&patient_recode_vec);
            //打印输出结果
            if let Some(allowed_set) = self.allow_record.get(&env::signer_account_id().as_str().to_string()) {
                // 将 UnorderedSet 转换为 Vec<String> 以便遍历
                let set_values: Vec<String> = allowed_set.to_vec();
            
                // 遍历并输出所有值
                for (index, value) in set_values.iter().enumerate() {
                    log_str(&format!("Index {}: Value: {}", index, value));
                }
            } else {
                // 处理键不存在的情况
                log_str("指定的键不存在");
            }
        }
        return true;
    }
    //查询某人病例，需要授权，自己查询自己病例除外
    // 查询用户所有病例的函数
    pub fn get_user_medical_records(&self, patient_id: String) -> Vec<MedicalRecord> {
        // 检查调用者是否是病人本人
        let caller = env::signer_account_id();
        let is_patient_himself = patient_id == caller.to_string();

        // 如果不是病人本人，则检查调用者是否在授权名单中
        if !is_patient_himself {
            assert!(self.allow_record.get(&patient_id).is_some() && self.allow_record.get(&patient_id).unwrap().contains(&caller.to_string()), "您没有权限查询该病人的病例");
        }

        // 获取患者的病例列表
        if let Some(patient_rec_vec) = self.patient_record.get(&patient_id) {
            // 返回病例列表
            return patient_rec_vec.to_vec();
        } else {
            return Vec::new();
        }
    }

}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use near_sdk::collections::vector;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    use super::*;
    //测试更变账户角色
    #[test]
    fn sign_test(){
        let mut contract = Contract::default();
        let mut role="doctor".to_string();
        let mut role1="patient".to_string();
        let judge=contract.register(role);
        let judge1=contract.register(role1);
        let my_role=contract.get_role();
        println!("{}",my_role);
    }
    //测试医生为病人添加病例
    #[test]
    fn add_record_terst(){
        let mut contract = Contract::default();
        let mut role="doctor".to_string();
        let judge=contract.register(role);
        let detail="病情无碍".to_string();
        let judge=contract.add_record("bob.near".to_string(),detail);
        
    }
    //测试添加授权
    #[test]
    fn allow_test(){
        let mut contract = Contract::default();
        
        let mut role1="patient".to_string();
        
        let judge1=contract.register(role1);
        let my_role=contract.add_allow_record("bob.near".to_string());
        println!("{}",my_role);
    }
    fn get_context(is_view: bool,user:String) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(user.parse().unwrap())
            .signer_account_id(user.parse().unwrap())
            .is_view(is_view)
            .build()
    }

    #[test]
    fn test_get_user_medical_records() {
        // 创建合约实例
        let mut contract = Contract::default();

        // 设置测试上下文，模拟医生查询病人的病例
        let context = get_context(false,"bob.near".to_string());
        testing_env!(context);

        // 假设这是一个病人ID和医生ID
        let patient_id = "bob.view".to_string();
        let doctor_id = "bob.near".to_string();
        contract.identify.insert(&patient_id,&"patient".to_string());
        contract.identify.insert(&doctor_id,&"doctor".to_string());
        // 假设这是一个病例详情
        let medical_record = MedicalRecord {
            doctor: doctor_id.clone(),
            detail: "Some details".to_string(),
            time: "2023-01-01".to_string(),
        };
        let mut test_vec:Vector<MedicalRecord>=Vector::new(b"n");
        test_vec.push(&medical_record);
        // 将病例添加到患者记录中
        contract.patient_record.insert(&patient_id, &test_vec);

        // 将医生添加到授权名单中
        let mut authorized_doctors = UnorderedSet::new(b"a".to_vec());
        authorized_doctors.insert(&doctor_id);
        contract.allow_record.insert(&patient_id, &authorized_doctors);

        // 以医生身份调用函数
        let patient_records = contract.get_user_medical_records(patient_id.clone());

        // 验证结果
        assert_eq!(patient_records.len(), 1);

        // 重置上下文为其他测试
        let context = get_context(true,"bob.near".to_string());
        testing_env!(context);
    }
}