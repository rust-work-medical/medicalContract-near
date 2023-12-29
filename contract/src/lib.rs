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
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
//药方记录
pub struct Prescription {
    pub medicine: Vec<Medicine>,
    pub prescribing_doctor: String,
    pub prescription_time: String,
    pub is_use:bool,
}
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
//药
pub struct Medicine {
    pub medicine_info: String,
    pub medicine_price: String,
    pub medicine_name: String,
}
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
//患者预约挂号记录
pub struct Reservation {
    pub patient: String,
    pub doctor: String,
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
    //病人药方记录
    pub patient_medicine: UnorderedMap<String, Vector<Prescription>>,
    //患者预约挂号记录列表
    pub reservation_record: UnorderedMap<String, Reservation>,
    //医生状态 true代表空闲 false代表已有预约
    pub doctor_status: UnorderedMap<String, bool>
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
            allow_record:UnorderedMap::new(b"a"),
            patient_medicine:UnorderedMap::new(b"m"),
            reservation_record:UnorderedMap::new(b"r"),
            doctor_status:UnorderedMap::new(b"d")
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    //注册
    pub fn register(&mut self,role:String) -> bool {
        self.identify.insert(&env::signer_account_id().as_str().to_string(),&role);
        if role == "doctor" {
            self.doctor_status.insert(&env::signer_account_id().as_str().to_string(),&true);
        }
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

    //检查某用户是否拥有某个权限
    pub fn check_role(&self, account_id: &str, role:String) -> bool {
        match self.identify.get(&account_id.to_string()){
            Some(x) => return x == role,
            None => panic!("该账户还没有角色")
        }
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
                            time: env::block_timestamp().to_string(),//timestamp?
                        };
                        //如果存在直接插入一条新记录，如果不存在则新建一个Vector,并插入
                        if self.patient_record.get(&patient_id).is_some() {
                            self.patient_record.get(&patient_id).unwrap().push(&medical_record);
                        }else {
                            let prefix: Vec<u8> = 
                            [
                                b"m".as_slice(),
                                &near_sdk::env::sha256_array(patient_id.as_bytes()),
                            ].concat();//prefix的作用?
                            let mut patient_recode_vec:Vector<MedicalRecord>=Vector::new(prefix);
                            patient_recode_vec.push(&medical_record);
                            self.patient_record.insert(&patient_id,&patient_recode_vec);
                        }
                        log_str(&format!("保存病例成功: {medical_record}"));
                        //TODO 这里的时间总是为0,不确定是不是测试环境导致的，后续还需要上链后测试
                        log_str(&format!("当前时间: {}",env::block_timestamp()));
                        return true;
                    },
                    None => return false,//panic
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
        if self.allow_record.get(&env::signer_account_id().as_str().to_string()).is_some() {
            self.allow_record.get(&env::signer_account_id().as_str().to_string()).unwrap().insert(&doctor);
        }else {
            let prefix: Vec<u8> = 
            [
                b"a".as_slice(),
                &near_sdk::env::sha256_array(&env::signer_account_id().as_str().to_string().as_bytes()),
            ].concat();//prefix的作用?
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
    //医生开药方
    pub fn prescribe_medicine(&mut self, patient_id: String, medicine: Vec<Medicine>) -> bool {
        // 获取调用者账户ID
        let doctor_id = env::signer_account_id().to_string();

        // 确保调用者是医生
        assert!(
            self.identify.get(&doctor_id) == Some("doctor".to_string()),
            "只有医生可以调用此函数"
        );

        // 创建处方记录
        let prescription = Prescription {
            medicine,
            prescribing_doctor: doctor_id.clone(),
            prescription_time: env::block_timestamp().to_string(),
            is_use: false,
        };

        // 将处方添加到患者记录中
        if self.patient_medicine.get(&patient_id).is_some(){
            self.patient_medicine.get(&patient_id).unwrap().push(&prescription);
        }else {
            let prefix: Vec<u8> = 
            [
                b"p".as_slice(),
                &near_sdk::env::sha256_array(&patient_id.as_bytes()),
            ].concat();
            let mut new_prescriptions: Vector<Prescription> = Vector::new(prefix);
            new_prescriptions.push(&prescription);
            self.patient_medicine.insert(&patient_id, &new_prescriptions);
        }

        true
    }
     // 药房工作人员确认用户缴费并发药
    pub fn confirm_payment_and_dispense_medicine(&mut self, patient_id: String, prescription_index: u64) -> bool {
        // 获取调用者账户ID
        let pharmacy_staff_id = env::signer_account_id().to_string();
        // 确保调用者是药房工作人员
        assert!(
            self.identify.get(&pharmacy_staff_id) == Some("pharmacy".to_string()),
            "只有药房工作人员可以调用此函数"
        );
        // 获取患者药方记录
        if self.patient_medicine.get(&patient_id).is_some() {
            // 确保索引有效
            assert!(prescription_index < self.patient_medicine.get(&patient_id).unwrap().len() as u64, "药方索引无效");
            // 获取要更新的药方记录
            let mut patient_prescription=self.patient_medicine.get(&patient_id).unwrap();
            if patient_prescription.get(prescription_index as u64).is_some() {
                // 直接更新 is_use 变量,必须使用replace，不能直接修改变量的值，是不起作用的
                let mut pre=patient_prescription.get(prescription_index as u64).unwrap();
                pre.is_use=true;
                patient_prescription.replace(prescription_index, &pre);
                return true;
            }
            return false;
        }
        false
    }
    // 查询某人药方的函数
    pub fn get_user_prescriptions(&self, patient_id: String) -> Vec<Prescription> {
        // 检查调用者是否是患者本人
        let caller = env::signer_account_id();
        let is_patient_himself = patient_id == caller.to_string();

        // 如果不是患者本人，则检查调用者是否在授权名单中
        if !is_patient_himself {
            assert!(
                self.allow_record.get(&patient_id).is_some()
                    && self.allow_record.get(&patient_id).unwrap().contains(&caller.to_string()),
                "您没有权限查询该患者的药方"
            );
        }

        // 获取患者的药方记录
        if let Some(patient_prescriptions) = self.patient_medicine.get(&patient_id) {
            // 返回药方记录列表
            return patient_prescriptions.to_vec();
        } else {
            return Vec::new();
        }
    }

    pub fn get_doctor_status(&self, doctor_id: &String) -> bool {
        if let Some(status) = self.doctor_status.get(doctor_id){
            status
        } else {
            panic!("invalid doctor id.");
        }
    }

    // 患者和处于空闲状态的医生预约挂号看病
    pub fn make_reservation(&mut self, doctor_id: &String) -> bool {
        let caller = env::signer_account_id().to_string();
        assert_eq!(self.check_role(&caller,"patient".to_string()),true,"Method caller isn't a patient.");
        assert_eq!(self.check_role(doctor_id,"doctor".to_string()),true,"Input doctor id isn't valid.");
        let mut status = self.get_doctor_status(doctor_id);
        if status == false {
            panic!("This doctor isn't avaliable.");
            return false;
        } else {
            status = false;
            self.doctor_status.insert(doctor_id,&status);
            let reservation_rec = Reservation {
                patient: caller.clone(),
                doctor: doctor_id.clone(),
                time: env::block_timestamp().to_string(),
            };
            self.reservation_record.insert(&caller, &reservation_rec);
        }
        true
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
    //建立测试环境
    fn set_context(is_view: bool,user:String) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(user.parse().unwrap());
        builder.signer_account_id(user.parse().unwrap());
        builder.is_view(is_view);
        testing_env!(builder.build());
    }

    //测试更变账户角色
    #[test]
    fn register_and_checkrole_test(){
        let mut contract = Contract::default();
        set_context(false,"alice.near".to_string());
        let mut doctor_id = "alice.near".to_string();
        let mut role = "doctor".to_string();
        let grant_role_status =  contract.register(role);
        assert_eq!(grant_role_status,true,"grant doctor role failed.");
        let check_role = contract.check_role(&doctor_id,"doctor".to_string());
        assert_eq!(check_role,true,"check doctor role failed.");

        set_context(false,"bob.near".to_string());
        let mut patient_id = "bob.near".to_string();
        let mut role2 = "patient".to_string();
        let grant_role2_status =  contract.register(role2);
        assert_eq!(grant_role2_status,true,"grant patient role failed.");
        let check_role2 = contract.check_role(&patient_id,"patient".to_string());
        assert_eq!(check_role2,true,"check patient role failed.");
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
    //测试添加药方，及验证药方及查询药方
    #[test]
    fn test_prescribe_medicine() {
        // 创建合约实例
        let mut contract = Contract::default();

        // 设置测试上下文，模拟医生调用开药方函数
        let context = get_context(false, "doctor.near".to_string());
        testing_env!(context);
        let doctor_id = "doctor.near".to_string();
        contract.identify.insert(&doctor_id,&"doctor".to_string());
        // 假设这是一个病人ID
        let patient_id = "patient.near".to_string();
        contract.identify.insert(&patient_id,&"patient".to_string());
        // 假设这是一个药物信息
        let medicine = vec![
            Medicine {
                medicine_info: "Information about Medicine A".to_string(),
                medicine_price: "10 NEAR".to_string(),
                medicine_name: "Medicine A".to_string(),
            },
            Medicine {
                medicine_info: "Information about Medicine B".to_string(),
                medicine_price: "15 NEAR".to_string(),
                medicine_name: "Medicine B".to_string(),
            },
        ];

        // 调用医生开药方函数
        let result = contract.prescribe_medicine(patient_id.clone(), medicine.clone());

        // 验证结果
        assert!(result, "开药方失败");

        // 验证药方是否正确添加到患者记录中
        let patient_medicine = contract.patient_medicine.get(&patient_id).unwrap();
        assert_eq!(patient_medicine.len(), 1, "患者记录中应有一条药方");

        let prescribed_medicine = &patient_medicine.get(0).unwrap();;
        assert_eq!(
            prescribed_medicine.medicine.len(),
            2,
            "药方中应包含两种药物"
        );
        //验证药方付款后修改使用状态
        contract.identify.insert(&doctor_id,&"pharmacy".to_string());

        // 重置上下文为其他测试
       
        let result = contract.confirm_payment_and_dispense_medicine(patient_id.clone(), 0);
         // 验证结果
         assert!(result);
       // 验证药方记录是否更新
        let updated_prescription = contract.patient_medicine.get(&patient_id).unwrap().get(0).unwrap();
        assert!(updated_prescription.is_use);

        // 额外检查：在修改后读取 is_use，并验证它是否被正确设置
        let is_use_after_update = updated_prescription.is_use;
        assert!(is_use_after_update);

        //测试查询用户药方情况
        // 以医生身份调用函数
        let mut authorized_doctors = UnorderedSet::new(b"a".to_vec());
        authorized_doctors.insert(&doctor_id);
        contract.allow_record.insert(&patient_id, &authorized_doctors);

        let patient_records = contract.get_user_prescriptions(patient_id.clone());

        // 验证结果
        assert_eq!(patient_records.len(), 1);

        let context = get_context(true, "doctor.near".to_string());
        testing_env!(context);
    }

    //测试预约挂号功能
    #[test]
    #[should_panic(expected = "This doctor isn't avaliable.")]
    fn reservation_test() {
        let mut contract = Contract::default();
        set_context(false,"doctor.near".to_string());
        let role = "doctor".to_string();
        let grant_role_status =  contract.register(role);

        set_context(false,"patient.near".to_string());
        let role2 = "patient".to_string();
        let grant_role2_status =  contract.register(role2);

        let doctor_id = "doctor.near".to_string();
        let status = contract.get_doctor_status(&doctor_id);
        assert_eq!(status,true,"doctor status invalid.");

        let reservation_result = contract.make_reservation(&doctor_id);
        assert_eq!(reservation_result,true,"reservation invalid.");

        let new_status = contract.get_doctor_status(&doctor_id);
        assert_eq!(new_status,false,"doctor status should be false.");

        let _ = contract.make_reservation(&doctor_id);//should panic
    }
}