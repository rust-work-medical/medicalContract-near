use std::collections::HashMap;
use std::ptr::null;
use std::fmt;
// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::log_str;
use near_sdk::serde::{Serialize,Deserialize};
use near_sdk::near_bindgen;
use near_sdk::{Balance,AccountId,env};
use near_sdk::collections::{LookupMap, Vector};

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
    identify:LookupMap<String, String>,
    //病例详情
    pub patient_record: LookupMap<String, Vector<MedicalRecord>>,
}
const MIN_STORAGE: Balance = 1_100_000_000_000_000_000_000_000; //1.1Ⓝ
const POINT_ONE: Balance = 100_000_000_000_000_000_000_000;//0.1N
// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self 
        {   
            //需要注意，每一个区块链接口提供的数据结构在初始化的时候都需要添加一个前缀，如果是嵌套结构，在嵌套结构里也需要添加前缀，前缀可以使用账户ID或者其他形式，前缀不能一样
            patient_record: LookupMap::new(b"p"),
            identify: LookupMap::new(b"i")
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
        assert!(self.identify.contains_key(&env::signer_account_id().as_str().to_string()), "该账户还没有角色");
        let val = match self.identify.get(&env::signer_account_id().as_str().to_string()) {
            Some(x) => return x,
            None => return  String::new(),
          };
    }
    //添加病例
    pub fn add_record(&mut self,patient:String,detail:String)->bool{
        assert!(self.identify.contains_key(&env::signer_account_id().as_str().to_string()), "该账户还没有角色");
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
                        if(self.patient_record.contains_key(&patient_id)){
                            self.patient_record.get(&patient_id).unwrap().push(&medical_record)
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
            None => return false,
          };
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
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
}