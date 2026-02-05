use std::{collections::HashMap, net::SocketAddr, path::PathBuf};

use just_enough_vcs::{
    lib::{
        data::{
            local::{
                LocalWorkspace,
                cached_sheet::CachedSheet,
                latest_file_data::LatestFileData,
                latest_info::LatestInfo,
                local_sheet::{LocalSheet, LocalSheetData},
                workspace_analyzer::{AnalyzeResult, AnalyzeResultPure},
                workspace_config::LocalConfig,
            },
            member::MemberId,
            sheet::{SheetData, SheetName},
            vault::vault_config::VaultUuid,
        },
        env::current_local_path,
    },
    utils::cfg_file::config::ConfigFile,
};

use crate::systems::cmd::errors::CmdPrepareError;

/// Temporarily enter a directory to execute a block of code, then return to the original directory
macro_rules! entry_dir {
    ($path:expr, $block:block) => {{
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir($path).unwrap();
        let result = $block;
        std::env::set_current_dir(original_dir).unwrap();
        result
    }};
}

#[derive(Default)]
pub struct LocalWorkspaceReader {
    cached_sheet: HashMap<SheetName, SheetData>,
    current_dir: Option<PathBuf>,
    workspace_dir: Option<PathBuf>,
    latest_file_data: HashMap<MemberId, LatestFileData>,
    latest_info: Option<LatestInfo>,
    local_config: Option<LocalConfig>,
    local_sheet_data: HashMap<(MemberId, SheetName), LocalSheetData>,
    local_workspace: Option<LocalWorkspace>,
}

impl LocalWorkspaceReader {
    // Get the current directory
    pub fn current_dir(&mut self) -> Result<&PathBuf, CmdPrepareError> {
        if self.current_dir.is_none() {
            let current_dir = std::env::current_dir()?;
            self.current_dir = Some(current_dir);
        }
        Ok(self.current_dir.as_ref().unwrap())
    }

    // Get the workspace directory
    pub fn workspace_dir(&mut self) -> Result<&PathBuf, CmdPrepareError> {
        if self.workspace_dir.is_none() {
            let workspace_dir = current_local_path();
            self.workspace_dir = workspace_dir;
            return match &self.workspace_dir {
                Some(d) => Ok(d),
                None => Err(CmdPrepareError::LocalWorkspaceNotFound),
            };
        }
        Ok(self.workspace_dir.as_ref().unwrap())
    }

    // Get the local configuration
    pub async fn local_config(&mut self) -> Result<&LocalConfig, CmdPrepareError> {
        if self.local_config.is_none() {
            let workspace_dir = self.workspace_dir()?;
            let local_config = entry_dir!(workspace_dir, {
                LocalConfig::read()
                    .await
                    .map_err(|_| CmdPrepareError::LocalConfigNotFound)?
            });
            self.local_config = Some(local_config)
        }
        Ok(self.local_config.as_ref().unwrap())
    }

    // Current account
    pub async fn current_account(&mut self) -> Result<MemberId, CmdPrepareError> {
        Ok(self.local_config().await?.current_account())
    }

    // Whether it is in host mode
    pub async fn is_host_mode(&mut self) -> Result<bool, CmdPrepareError> {
        Ok(self.local_config().await?.is_host_mode())
    }

    // Whether the workspace is stained
    pub async fn workspace_stained(&mut self) -> Result<bool, CmdPrepareError> {
        Ok(self.local_config().await?.stained())
    }

    // Stain UUID
    pub async fn stained_uuid(&mut self) -> Result<Option<VaultUuid>, CmdPrepareError> {
        Ok(self.local_config().await?.stained_uuid())
    }

    // Upstream address
    pub async fn upstream_addr(&mut self) -> Result<SocketAddr, CmdPrepareError> {
        Ok(self.local_config().await?.upstream_addr())
    }

    // Currently used sheet
    pub async fn sheet_in_use(&mut self) -> Result<&Option<SheetName>, CmdPrepareError> {
        Ok(self.local_config().await?.sheet_in_use())
    }

    // Get the sheet name in use, or error if none
    pub async fn sheet_name(&mut self) -> Result<SheetName, CmdPrepareError> {
        match self.local_config().await?.sheet_in_use() {
            Some(name) => Ok(name.clone()),
            None => Err(CmdPrepareError::NoSheetInUse),
        }
    }

    // Current draft folder
    pub async fn current_draft_folder(&mut self) -> Result<Option<PathBuf>, CmdPrepareError> {
        Ok(self.local_config().await?.current_draft_folder())
    }

    // Get the local workspace
    pub async fn local_workspace(&mut self) -> Result<&LocalWorkspace, CmdPrepareError> {
        if self.local_workspace.is_none() {
            let workspace_dir = self.workspace_dir()?.clone();
            let local_config = self.local_config().await?.clone();
            let Some(local_workspace) = entry_dir!(&workspace_dir, {
                LocalWorkspace::init_current_dir(local_config)
            }) else {
                return Err(CmdPrepareError::LocalWorkspaceNotFound);
            };
            self.local_workspace = Some(local_workspace);
        }
        Ok(self.local_workspace.as_ref().unwrap())
    }

    // Get the latest information
    pub async fn latest_info(&mut self) -> Result<&LatestInfo, CmdPrepareError> {
        if self.latest_info.is_none() {
            let local_dir = self.workspace_dir()?.clone();
            let local_config = self.local_config().await?.clone();
            let latest_info = entry_dir!(&local_dir, {
                match LatestInfo::read_from(LatestInfo::latest_info_path(
                    &local_dir,
                    &local_config.current_account(),
                ))
                .await
                {
                    Ok(info) => info,
                    Err(_) => {
                        return Err(CmdPrepareError::LatestInfoNotFound);
                    }
                }
            });
            self.latest_info = Some(latest_info);
        }
        Ok(self.latest_info.as_ref().unwrap())
    }

    // Get the latest file data
    pub async fn latest_file_data(
        &mut self,
        account: &MemberId,
    ) -> Result<&LatestFileData, CmdPrepareError> {
        if !self.latest_file_data.contains_key(account) {
            let local_dir = self.workspace_dir()?;
            let latest_file_data_path =
                match entry_dir!(&local_dir, { LatestFileData::data_path(account) }) {
                    Ok(p) => p,
                    Err(_) => return Err(CmdPrepareError::LatestFileDataNotExist(account.clone())),
                };
            let latest_file_data = LatestFileData::read_from(&latest_file_data_path).await?;
            self.latest_file_data
                .insert(account.clone(), latest_file_data);
        }

        Ok(self.latest_file_data.get(account).unwrap())
    }

    // Get the cached sheet
    pub async fn cached_sheet(
        &mut self,
        sheet_name: &SheetName,
    ) -> Result<&SheetData, CmdPrepareError> {
        if !self.cached_sheet.contains_key(sheet_name) {
            let workspace_dir = self.workspace_dir()?;
            let cached_sheet = entry_dir!(&workspace_dir, {
                match CachedSheet::cached_sheet_data(sheet_name).await {
                    Ok(data) => data,
                    Err(_) => return Err(CmdPrepareError::CachedSheetNotFound(sheet_name.clone())),
                }
            });
            self.cached_sheet.insert(sheet_name.clone(), cached_sheet);
        }

        Ok(self.cached_sheet.get(sheet_name).unwrap())
    }

    // Get the local sheet data
    pub async fn local_sheet_data(
        &mut self,
        account: &MemberId,
        sheet_name: &SheetName,
    ) -> Result<&LocalSheetData, CmdPrepareError> {
        let key = (account.clone(), sheet_name.clone());
        if !self.local_sheet_data.contains_key(&key) {
            let workspace_dir = self.workspace_dir()?.clone();
            let local_workspace = self.local_workspace().await?;
            let path = entry_dir!(&workspace_dir, {
                local_workspace.local_sheet_path(account, sheet_name)
            });
            let local_sheet_data = match LocalSheetData::read_from(path).await {
                Ok(data) => data,
                Err(_) => {
                    return Err(CmdPrepareError::LocalSheetNotFound(
                        account.clone(),
                        sheet_name.clone(),
                    ));
                }
            };
            self.local_sheet_data.insert(key.clone(), local_sheet_data);
        }

        Ok(self.local_sheet_data.get(&key).unwrap())
    }

    // Clone and get the local sheet
    pub async fn local_sheet_cloned(
        &mut self,
        account: &MemberId,
        sheet_name: &SheetName,
    ) -> Result<LocalSheet<'_>, CmdPrepareError> {
        let local_sheet_data = self.local_sheet_data(account, sheet_name).await?.clone();
        Ok(LocalSheet::new(
            self.local_workspace().await?,
            account.clone(),
            sheet_name.clone(),
            local_sheet_data,
        ))
    }

    // Analyze local status
    pub async fn analyze_local_status(&mut self) -> Result<AnalyzeResultPure, CmdPrepareError> {
        let Ok(analyzed) = AnalyzeResult::analyze_local_status(self.local_workspace().await?).await
        else {
            return Err(CmdPrepareError::LocalStatusAnalyzeFailed);
        };
        Ok(analyzed.into())
    }

    // Pop the local configuration (take ownership if cached)
    pub async fn pop_local_config(&mut self) -> Result<LocalConfig, CmdPrepareError> {
        if let Some(local_config) = self.local_config.take() {
            Ok(local_config)
        } else {
            let workspace_dir = self.workspace_dir()?;
            let local_config = entry_dir!(workspace_dir, {
                LocalConfig::read()
                    .await
                    .map_err(|_| CmdPrepareError::LocalConfigNotFound)?
            });
            Ok(local_config)
        }
    }

    // Pop the local workspace (take ownership if cached)
    pub async fn pop_local_workspace(&mut self) -> Result<LocalWorkspace, CmdPrepareError> {
        if let Some(local_workspace) = self.local_workspace.take() {
            Ok(local_workspace)
        } else {
            let workspace_dir = self.workspace_dir()?.clone();
            let local_config = self.local_config().await?.clone();
            let Some(local_workspace) = entry_dir!(&workspace_dir, {
                LocalWorkspace::init_current_dir(local_config)
            }) else {
                return Err(CmdPrepareError::LocalWorkspaceNotFound);
            };
            Ok(local_workspace)
        }
    }

    // Pop the latest information (take ownership if cached)
    pub async fn pop_latest_info(&mut self) -> Result<LatestInfo, CmdPrepareError> {
        if let Some(latest_info) = self.latest_info.take() {
            Ok(latest_info)
        } else {
            let local_dir = self.workspace_dir()?.clone();
            let local_config = self.local_config().await?.clone();
            let latest_info = entry_dir!(&local_dir, {
                match LatestInfo::read_from(LatestInfo::latest_info_path(
                    &local_dir,
                    &local_config.current_account(),
                ))
                .await
                {
                    Ok(info) => info,
                    Err(_) => {
                        return Err(CmdPrepareError::LatestInfoNotFound);
                    }
                }
            });
            Ok(latest_info)
        }
    }

    // Pop the latest file data for a specific account (take ownership if cached)
    pub async fn pop_latest_file_data(
        &mut self,
        account: &MemberId,
    ) -> Result<LatestFileData, CmdPrepareError> {
        if let Some(latest_file_data) = self.latest_file_data.remove(account) {
            Ok(latest_file_data)
        } else {
            let local_dir = self.workspace_dir()?;
            let latest_file_data_path =
                match entry_dir!(&local_dir, { LatestFileData::data_path(account) }) {
                    Ok(p) => p,
                    Err(_) => return Err(CmdPrepareError::LatestFileDataNotExist(account.clone())),
                };
            let latest_file_data = LatestFileData::read_from(&latest_file_data_path).await?;
            Ok(latest_file_data)
        }
    }

    // Pop the cached sheet for a specific sheet name (take ownership if cached)
    pub async fn pop_cached_sheet(
        &mut self,
        sheet_name: &SheetName,
    ) -> Result<SheetData, CmdPrepareError> {
        if let Some(cached_sheet) = self.cached_sheet.remove(sheet_name) {
            Ok(cached_sheet)
        } else {
            let workspace_dir = self.workspace_dir()?;
            let cached_sheet = entry_dir!(&workspace_dir, {
                match CachedSheet::cached_sheet_data(sheet_name).await {
                    Ok(data) => data,
                    Err(_) => return Err(CmdPrepareError::CachedSheetNotFound(sheet_name.clone())),
                }
            });
            Ok(cached_sheet)
        }
    }

    // Pop the local sheet data for a specific account and sheet name (take ownership if cached)
    pub async fn pop_local_sheet_data(
        &mut self,
        account: &MemberId,
        sheet_name: &SheetName,
    ) -> Result<LocalSheetData, CmdPrepareError> {
        let key = (account.clone(), sheet_name.clone());
        if let Some(local_sheet_data) = self.local_sheet_data.remove(&key) {
            Ok(local_sheet_data)
        } else {
            let workspace_dir = self.workspace_dir()?.clone();
            let local_workspace = self.local_workspace().await?;
            let path = entry_dir!(&workspace_dir, {
                local_workspace.local_sheet_path(account, sheet_name)
            });
            let local_sheet_data = match LocalSheetData::read_from(path).await {
                Ok(data) => data,
                Err(_) => {
                    return Err(CmdPrepareError::LocalSheetNotFound(
                        account.clone(),
                        sheet_name.clone(),
                    ));
                }
            };
            Ok(local_sheet_data)
        }
    }
}
