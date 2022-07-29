use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{log, env, near_bindgen, AccountId, Promise};
//Función que nos regresa el valor de 1 NEAR en un u128
fn one_near() -> u128 {
    u128::from_str_radix("1000000000000000000000000", 10).unwrap()
}

//Definimos el struct principal.
//Si nuestro contrato necesitara más colecciones, estas se definen aquí.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FContract {
    farmers: UnorderedMap<String, Farmer>,
    balance: u64,
    rewards: u64,
}

impl Default for FContract {
    fn default() -> Self {
        Self {
            //Inicializamos la colección con un prefijo único
            farmers: UnorderedMap::new(b"f".to_vec()),
            balance: 0,
            rewards: 0,
        }
    }
}

//Definimos los structs que utilizaremos dentro del contrato
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Farmer {
    pub account: String,
    pub amount: u64,
}

//En este contrato no se utiliza el default, pero es buena práctica tenerlo inicializado.
impl Default for Farmer {
    fn default() -> Self {
        Farmer {
            account: String::from(""),
            amount: 0,
        }
    }
}

//Creamos la implementación del método new. El equivalente en AS sería el constructor.
impl Farmer {
    pub fn new(account: String, amount: u64) -> Self {
        Self {
            account,
            amount,
        }
    }
}

//Igual que con el struct de Farmer, implementamos los métodos del contrato en un impl.
#[near_bindgen]
impl FContract {    

    /// Método de ESCRITURA para registrar un nuevo deposito
    /// El comando para utilizarlo en la terminal es:
    /// >> near call $CONTRATO set_deposit '{"amount":10}' --amount 10 --accountId CUENTA.testnet
    ///  $CONTRATO es una variable que contiene el id de la cuenta del contrato
    /// @param amount entero de 64 bits sin signo que representa el deposito del usuario dentro del pool
    #[payable]
    pub fn set_deposit(&mut self, amount: u64) {
        let account = env::signer_account_id().to_string();
        let deposit = env::attached_deposit();

        assert!(deposit > 0, "Importe inválido, debe ser mayor a 0.");

        let farmer = Farmer::new(account.clone(), amount);
        self.farmers.insert(&account, &farmer);
        log!("deposit  {}", deposit);
        log!("self.balance 1 {}", self.balance);
        self.balance += amount;
        log!("self.balance 2 {}", self.balance);
        env::log_str("Depósito realizado exitosamente.");
    }

    #[payable]
    pub fn set_deposit_team(&mut self, amount: u64) {
        let master: AccountId = "team12.testnet".parse().unwrap();
        let deposit = env::attached_deposit();

        assert!(
            env::signer_account_id() == master,
            "No tienes permisos para depositar recompensas."
        );        

        assert!(
            self.balance > 0,
            "Debe haber depositos en el pool antes de cargar las recompensas."
        );        

        assert!(deposit > 0, "Importe inválido, debe ser mayor a 0.");

        self.rewards += amount;

        env::log_str("Depósito realizado exitosamente.");
    }    

    /// Método de LECTURA que regresa un depositante
    /// El comando para utilizarlo en la terminal es:
    /// >> near view $CONTRATO get_farmer '{"account":"CUENTA.testnet"}'
    /// @param account string que contiene la cuenta (key) del depositante a consultar
    /// @returns Option<Farmer>
    pub fn get_farmer(&self, account: String) -> Option<Farmer> {
        self.farmers.get(&account)
    }

    /// Método de LECTURA que regresa toda la lista de los depositantes del pool
    /// El comando para utilizarlo en la terminal es:
    ///  >> near view $CONTRATO get_farmers '{}'
    /// @returns Vec<Farmer> (vector de depositantes)
    pub fn get_farmers(&self) -> Vec<Farmer> {
        self.farmers.values_as_vector().to_vec()
    }

    /// Método de LECTURA que regresa el total de lo depositado en el pool
    /// El comando para utilizarlo en la terminal es:
    ///  >> near view $CONTRATO get_balance '{}'
    /// @returns Vec<Farmer> (vector de depositantes)
    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    /// Método de LECTURA que regresa el total de las recompensas en el pool
    /// El comando para utilizarlo en la terminal es:
    ///  >> near view $CONTRATO get_rewards '{}'
    /// @returns Vec<Farmer> (vector de depositantes)
    pub fn get_rewards(&self) -> u64 {
        self.rewards
    }    

    /// Método de ESCRITURA para realizar el retiro de los fondos de un depositante
    /// El comando para utilizarlo en la terminal es:
    ///  >> near call $CONTRATO set_withdrawal '{"account":"CUENTA.testnet","amount":20}' --accountId CUENTA.testnet
    /// @param account string que contiene la cuenta del participante que desea retirarse del pool
    /// @param amount u64 que contiene del monto que desea retirar del pool
    /// @returns bool: Regresa verdadero o falso dependiendo de si se ejecutó la acción.
    pub fn set_withdrawal(&mut self, account: String, amount: u64) -> bool {
        let near_account: AccountId = account.parse().unwrap();

        assert!(
            env::signer_account_id() == near_account,
            "No tiene permiso para realizar este retiro."
        );

        assert!(
            self.rewards > 0,
            "No es posible retirar del pool antes de que el Team deposite las recompensas"
        );

        match self.farmers.get(&account) {
            Some(mut farmer) => {
                
                assert!(
                    farmer.amount >= amount,
                    "No es posible retirar un monto mayor al ingresado en el pool"
                );

                let share: f64 =  farmer.amount as f64 / self.balance as f64;
                log!("self.balance {}", self.balance);
                log!("self.rewards {}", self.rewards);
                log!("share {}", share);
                let farmer_rewards = self.rewards as f64 * share as f64;
                log!("farmer_rewards {}", farmer_rewards);
                let transfer_amount = farmer.amount as f64 + farmer_rewards.round();
                farmer.amount = farmer.amount - amount;
                log!("transfer_amount {}", transfer_amount.round());

                let transfer_amount_o = one_near()
                .checked_mul(transfer_amount as u128)
                .unwrap_or(0);
                Promise::new(near_account).transfer(transfer_amount_o);
                self.farmers.insert(&account, &farmer);
                self.balance -= amount;
                self.rewards -= farmer_rewards as u64;
                log!("transfer_amount_o {}", transfer_amount_o);
                log!("self.balance {}", self.balance);
                log!("self.rewards {}", self.rewards);
                env::log_str("Retiro realizado existosamente");

                true
            }
            None => {
                env::log_str("Depositante no encontrado.");
                false
            }
        }
    }
}

// PRUEBAS UNITARIAS
// Para correr las pruebas unitarias ejecuta el comando: cargo test
// Puedes encontrar más información en: https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    use super::*;

    const CUENTA: &str = "team12.testnet";
    const DEPOSITANTE: &str = "team12.testnet";
    const BALANCE: u64 = 1;
    const DEPOSITO: u64 = 1;
    const REWARDS: u64 = 1;
    const RETIRO: u64 = 1;

    fn set_context() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());

        testing_env!(context
            .attached_deposit(DEPOSITO.into())
            .signer_account_id(CUENTA.parse().unwrap())
            .build());
    }

    #[test]
    pub fn test_set_deposit() {
        set_context();
        let mut contrato = FContract::default();

        contrato.set_deposit(DEPOSITO);
    }

    #[test]
    pub fn set_deposit_team() {
        set_context();
        let mut contrato = FContract::default();
        contrato.balance = BALANCE;

        contrato.set_deposit_team(DEPOSITO);
    }

    #[test]
    pub fn set_withdrawal() {
        set_context();
        let mut contrato = FContract::default();
        contrato.rewards = REWARDS;

        contrato.set_withdrawal(DEPOSITANTE.to_string(), RETIRO);
    }
}