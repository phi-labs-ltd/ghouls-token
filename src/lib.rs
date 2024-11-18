use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Empty, Reply, SubMsgResult};
use cw2::{get_contract_version, set_contract_version};
pub use cw721_base_updatable::{ContractError, InstantiateMsg, MintMsg, MinterResponse, QueryMsg};
use cw721_metadata::Metadata;
use cw721_updatable::ContractInfoResponse;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

pub type Extension = Option<Metadata>;

pub type Cw721MetadataContract<'a> =
    cw721_base_updatable::Cw721Contract<'a, Extension, Empty, Empty, Empty>;

pub type ExecuteMsg = cw721_base_updatable::ExecuteMsg<Extension, Empty>;
pub type UpdateMetadataMsg = cw721_base_updatable::msg::UpdateMetadataMsg<Extension>;

const CONTRACT_NAME: &str = "ghouls-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod entry {
    use super::*;

    #[cfg(not(feature = "library"))]
    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let info = ContractInfoResponse {
            name: msg.name,
            symbol: msg.symbol,
        };
        Cw721MetadataContract::default()
            .contract_info
            .save(deps.storage, &info)?;
        let minter = deps.api.addr_validate(&msg.minter)?;
        Cw721MetadataContract::default()
            .minter
            .save(deps.storage, &minter)?;
        Ok(Response::default())
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        Cw721MetadataContract::default().execute(deps, env, info, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
        match msg.result {
            SubMsgResult::Ok(_) => Ok(Response::default()),
            SubMsgResult::Err(_) => Err(ContractError::Unauthorized {}),
        }
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg<Empty>) -> StdResult<Binary> {
        Cw721MetadataContract::default().query(deps, env, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
        let original_version = get_contract_version(deps.storage)?;
        let name = CONTRACT_NAME.to_string();
        let version = CONTRACT_VERSION.to_string();
        if original_version.contract != name {
            return Err(ContractError::Unauthorized {});
        }
        if original_version.version >= version {
            return Err(ContractError::Unauthorized {});
        }

        // BEGIN v0.1.1 State Migration
        let info = ContractInfoResponse {
            name: "Ghouls".to_string(),
            symbol: "GHOUL".to_string(),
        };
        Cw721MetadataContract::default()
            .contract_info
            .save(deps.storage, &info)?;
        // END v0.1.1 State Migration

        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        Ok(Response::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721_metadata::Attribute;
    use cw721_updatable::{Cw721Query, NftInfoResponse};

    const CREATOR: &str = "creator";

    #[test]
    fn use_metadata_extension() {
        let mut deps = mock_dependencies();
        let contract = Cw721MetadataContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "NFToken".to_string(),
            symbol: "NFT".to_string(),
            minter: CREATOR.to_string(),
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        let attributes: Vec<Attribute> = vec![Attribute {
            trait_type: Some("Accessory".to_string()),
            value: Some("Earrings".to_string()),
        }];

        let metadata_extension = Some(Metadata {
            name: Some("NFT 1".into()),
            description: Some("default token description".into()),
            image: Some("ipfs://QmZdPdZzZum2jQ7jg1ekfeE3LSz1avAaa42G6mfimw9TEn".into()),
            attributes: Some(attributes),
            animation_url: None,
            external_url: None,
            properties: None,
        });

        let token_id = "1";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: CREATOR.to_string(),
            token_uri: None,
            extension: metadata_extension,
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        contract
            .execute(deps.as_mut(), mock_env(), info, exec_msg)
            .unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();

        assert_eq!(res.token_uri, mint_msg.token_uri);
        assert_eq!(res.extension, mint_msg.extension);
    }

    #[test]
    fn updating_metadata() {
        let mut deps = mock_dependencies();
        let contract = Cw721MetadataContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "NFToken".to_string(),
            symbol: "NFT".to_string(),
            minter: CREATOR.to_string(),
        };
        contract
            .instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg)
            .unwrap();

        let token_id1 = "updatable".to_string();
        let token_id2 = "won't be updated".to_string();

        let attributes: Vec<Attribute> = vec![Attribute {
            trait_type: Some("Accessory".to_string()),
            value: Some("Earrings".to_string()),
        }];

        let metadata_extension = Some(Metadata {
            name: Some("NFT 1".into()),
            description: Some("default token description".into()),
            image: Some("ipfs://QmZdPdZzZum2jQ7jg1ekfeE3LSz1avAaa42G6mfimw9TEn".into()),
            attributes: Some(attributes.clone()),
            animation_url: None,
            external_url: None,
            properties: None,
        });

        let modified_metadata_extension = Some(Metadata {
            name: Some("Modified NFT 1".into()),
            description: Some("Modified token description".into()),
            image: Some("ipfs://QmZdPdZzZum2jQ7jg1ekfeE3LSz1avAaa42G6mfimw9TEn".into()),
            attributes: Some(attributes),
            animation_url: None,
            external_url: None,
            properties: None,
        });

        let mint_msg = ExecuteMsg::Mint(MintMsg {
            token_id: token_id1.clone(),
            owner: CREATOR.to_string(),
            token_uri: None,
            extension: metadata_extension.clone(),
        });

        let mint_msg2 = ExecuteMsg::Mint(MintMsg {
            token_id: token_id2.clone(),
            owner: "innocent hodlr".to_string(),
            token_uri: None,
            extension: metadata_extension.clone(),
        });

        let err_metadata_extension = Some(Metadata {
            name: Some("evil doer".into()),
            description: Some("has rugged your token".into()),
            image: Some("rugged".into()),
            attributes: None,
            animation_url: None,
            external_url: None,
            properties: None,
        });

        let update_msg = ExecuteMsg::UpdateMetadata(UpdateMetadataMsg {
            token_id: token_id1.clone(),
            extension: modified_metadata_extension.clone(),
        });

        let err_update_msg = ExecuteMsg::UpdateMetadata(UpdateMetadataMsg {
            token_id: token_id1.clone(),
            extension: err_metadata_extension.clone(),
        });

        // Mint
        let admin = mock_info(CREATOR, &[]);
        let _mint1 = contract
            .execute(deps.as_mut(), mock_env(), admin.clone(), mint_msg)
            .unwrap();

        let _mint2 = contract
            .execute(deps.as_mut(), mock_env(), admin.clone(), mint_msg2)
            .unwrap();

        // Original NFT infos are correct
        let info1 = contract.nft_info(deps.as_ref(), token_id1.clone()).unwrap();
        assert_eq!(
            info1,
            NftInfoResponse {
                token_uri: None,
                extension: metadata_extension.clone(),
            }
        );

        let info2 = contract.nft_info(deps.as_ref(), token_id2.clone()).unwrap();
        assert_eq!(
            info2,
            NftInfoResponse {
                token_uri: None,
                extension: metadata_extension.clone(),
            }
        );

        // Random cannot update NFT
        let random = mock_info("random", &[]);

        let err = contract
            .execute(deps.as_mut(), mock_env(), random, err_update_msg)
            .unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // Only allowed minters can update NFT
        let _update = contract
            .execute(deps.as_mut(), mock_env(), admin.clone(), update_msg)
            .unwrap();

        let update_info = contract.nft_info(deps.as_ref(), token_id1.clone()).unwrap();

        // Modified NFT info is correct
        assert_eq!(
            update_info,
            NftInfoResponse {
                token_uri: None,
                extension: modified_metadata_extension,
            }
        );
    }
}
