use crate::config::DBConfig;
use bytebuffer::ByteBuffer;
//use serde::Deserialize;
use crate::page::PageId;
use bincode;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::{Read, Seek, SeekFrom, Write};

pub struct DiskManager<'a> {
    config: &'a DBConfig,

    free_pages: Vec<PageId>, //Vecteur de pages libres courante
}

impl<'a> DiskManager<'a> {
    pub fn new(config: &'a DBConfig) -> Self {
        let mut dm = Self {
            config,                 // Assignation de la référence de configuration
            free_pages: Vec::new(), // Initialisation du vecteur de pages libres
        };

        if let Err(e) = dm.load_state() {
            eprintln!("Erreur lors du chargement de l'état : {}", e);
            // Ici, tu peux soit retourner une valeur par défaut, soit panique.
            // Par exemple, on peut panique si c'est critique :
            panic!("Échec de l'allocation de la page : {}", e);
        } // Charger l'état initial des pages libres

        dm // Retourner l'instance de DiskManager
    }

    pub fn alloc_page(&mut self) -> PageId {
        self.free_pages.clear();
        if let Err(e) = self.load_state() {
            eprintln!("Erreur lors du chargement de l'état : {}", e);
            // Ici, tu peux soit retourner une valeur par défaut, soit panique.
            // Par exemple, on peut panique si c'est critique :
            panic!("Échec de l'allocation de la page : {}", e);
        }

        while let Some(page_id) = self.free_pages.pop() {
            if !self.page_exists(&page_id) {
                continue;
            }
            if let Err(e) = self.save_state() {
                eprintln!("Erreur lors de la sauvegarde de l'état : {}", e);
                // Tu peux choisir de panique ici aussi, ou de gérer l'erreur d'une autre manière.
                panic!("Échec de l'allocation de la page : {}", e);
            }

            return page_id;
        }

        let mut file_idx = 0;

        loop {
            let file_path = format!("{}/BinData/F{}.rsdb", self.config.get_dbpath(), file_idx);

            // Créer les répertoires si nécessaire
            if let Some(parent_dir) = std::path::Path::new(&file_path).parent() {
                fs::create_dir_all(parent_dir)
                    .expect("Impossible de créer les répertoires parent.");
            }

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&file_path)
                .unwrap();

            let current_size = file.metadata().unwrap().len() as u32;
            let page_size = self.config.get_page_size();
            let max_file_size = self.config.get_dm_maxfilesize();

            if current_size < max_file_size {
                let new_page_id = PageId::new(file_idx, current_size / page_size);

                let forbidden_value = 0xFF; // On écrit dans la page une valeur interdite pour marquer la présence.
                let mut write_buffer = Vec::<u8>::new();

                let byte_array = vec![forbidden_value; self.config.get_page_size() as usize];
                write_buffer.extend_from_slice(byte_array.as_ref());

                // Écriture du contenu du tampon dans le fichier
                if let Err(e) = file.write_all(&write_buffer) {
                    eprintln!("Erreur lors de l'écriture dans le fichier : {}", e);
                    panic!("Échec de l'écriture de la page : {}", e); // Ou gère l'erreur d'une autre manière
                }

                return new_page_id;
            }

            file_idx += 1;
        }
    }

    fn page_exists(&self, page_id: &PageId) -> bool {
        let file_path = format!(
            "{}/BinData/F{}.rsdb",
            self.config.get_dbpath(),
            page_id.get_file_idx()
        );
        let page_end = (page_id.get_page_idx() + 1) * self.config.get_page_size();

        std::fs::metadata(file_path)
            .map(|metadata| metadata.len() as u32 >= page_end)
            .unwrap_or(false)
    }

    //SI PAS COMPRIS, IL FAUT DEMANDER À MATHIEU
    pub fn read_page(&self, page_id: &PageId, buff: &mut ByteBuffer) -> Result<(), std::io::Error> {
        //vérifier si page existe
        let num_fichier = page_id.get_file_idx();
        let num_page = page_id.get_page_idx();

        //Ouverture du fichier
        let mut fichier: File = OpenOptions::new().read(true).open(format!(
            "{}/BinData/F{}.rsdb",
            self.config.get_dbpath(),
            num_fichier
        ))?;

        //Placement du pointeur dans le fichier
        fichier.seek(SeekFrom::Start(
            (num_page * self.config.get_page_size()) as u64,
        ))?;

        //Creation d'un buffer temporaire pour stocker les données
        let mut temp_buffer: Vec<u8> = vec![0; self.config.get_page_size() as usize];

        //Lecture des données
        fichier.read_exact(&mut temp_buffer)?;

        //Ecriture des données dans le buffer
        buff.write_bytes(&temp_buffer);

        //Affichage du buffer
        Ok(())
    }

    //SI PAS COMPRIS, IL FAUT DEMANDER À MATHIEU
    pub fn write_page(
        &self,
        page_id: &PageId,
        buff: &mut ByteBuffer,
    ) -> Result<(), Box<dyn Error>> {
        //faudrait vérifier que la page est libre je pense
        let num_fichier = page_id.get_file_idx();
        let num_page = page_id.get_page_idx();

        //Ouverture du fichier avec les droits d'écriture et d'ajout
        let mut fichier: File = OpenOptions::new().write(true).append(false).open(format!(
            "{}/BinData/F{}.rsdb",
            self.config.get_dbpath(),
            num_fichier
        ))?;

        //placement du pointeur dans le fichier
        fichier.seek(SeekFrom::Start(
            (num_page * self.config.get_page_size()) as u64,
        ))?; //a faire aorès pour le ?

        let data_to_write = buff.as_bytes();
        //Ecriture des données dans le fichier
        //URGENT A DEMANDER A MATHIEU
        //fichier.write_all(&buff.read_bytes(self.config.get_page_size() as usize)?)?;// Use the write_all method //transformer le truc en vecteur avant de faire le write_all
        fichier.write_all(&data_to_write)?;

        Ok(())
        //Utiliser le crate buffer avec un reader.
    }

    pub fn save_state(&self) -> std::io::Result<()> {
        let dm_save_path = format!("{}/dm.save", self.config.get_dbpath());

        // Supprimer le fichier s'il existe, a priori ca marche sans maintenant mais dans le doute.
        let _ = std::fs::remove_file(&dm_save_path);

        // Ouvrir le fichier avec l'option de création s'il n'existe pas
        let fichier = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(dm_save_path)?;

        // Utiliser BufWriter pour améliorer les performances lors de multiples écritures
        let mut writer = BufWriter::new(fichier);

        //On doit itérer 1 par 1 car si on sauvegarde tout le vecteur d'un coups, le premier élément sérialisé est la taille
        //Donc ca fausse notre manière de faire.
        // Itérer sur chaque élément du vecteur self.free_pages
        for page in &self.free_pages {
            // Sérialiser chaque page individuellement.
            //LIGNE OBSOLÈTE À CHANGER, MAIS JE VOUS CACHE PAS JE SUIS FATIGUÉ + FLEMME
            let contenu_sérialisé = bincode::serialize(&page)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            // Écrire l'élément sérialisé dans le fichier
            writer.write_all(&contenu_sérialisé)?;
        }

        // Tout s'est bien passé
        Ok(())
    }

    pub fn load_state(&mut self) -> std::io::Result<()> {
        let dm_save_path = format!("{}/dm.save", self.config.get_dbpath());

        let mut fichier = OpenOptions::new()
            .read(true) // Permet la lecture
            .write(true) // Permet l'écriture
            .create(true) // Crée le fichier s'il n'existe pas
            .open(dm_save_path)
            .expect("Impossible d'ouvrir ou de créer le fichier");

        let mut contenu = Vec::new();
        fichier.read_to_end(&mut contenu).expect("tkt");

        //self.free_pages.clear();

        let mut pos = 0;

        while pos < contenu.len() {
            match bincode::deserialize::<PageId>(&contenu[pos..]) {
                Ok(instance) => {
                    self.free_pages.push(instance);

                    pos += bincode::serialized_size(&self.free_pages.last().unwrap()).unwrap()
                        as usize;
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    pub fn dealloc_page(&mut self, page_id: PageId) {
        if !self.free_pages.contains(&page_id) {
            self.free_pages.push(page_id);
            if let Err(e) = self.save_state() {
                eprintln!("Erreur lors de la sauvegarde de l'état : {}", e);
                // Tu peux choisir de panique ici aussi, ou de gérer l'erreur d'une autre manière.
                panic!("Échec de l'allocation de la page : {}", e);
            }
        }
    }

    pub fn get_dbconfig(&self) -> &DBConfig {
        return &self.config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constructeur() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);
        assert_eq!(dm.config.get_dbpath(), "res/dbpath");
    }

    #[test]
    fn test_write_page_and_read_page_and_alloc_page() {
        let config = DBConfig::load_db_config("config.json".to_string());
        let mut dm = DiskManager::new(&config);
        let page_id = dm.alloc_page(); //PageId::new(999,0);
                                       //TEST ÉCRITURE
        let mut write_buffer = ByteBuffer::new();
        let byte_array = [7; 32];
        write_buffer.write_bytes(byte_array.as_ref());

        dm.write_page(&page_id, &mut write_buffer)
            .expect("write_page failed");

        //TEST LECTURE
        let mut read_buff = ByteBuffer::new();
        dm.read_page(&page_id, &mut read_buff)
            .expect("read_page failed");

        let expected_data = [7; 32]; //PASSER LES BITS À 1 POUR FAIRE ÉCHOUER LE TEST
        let read_data = read_buff.as_bytes();

        //TEST QUE LES DONNÉES ÉCRITE ET LUE SONT PAREILS
        assert_eq!(&read_data[..expected_data.len()], &expected_data[..]);
    }

    #[test]
    fn test_alloc_page() {
        let config = DBConfig::load_db_config("config.json".to_string());
        let mut dm = DiskManager::new(&config);
        let _page_id = dm.alloc_page();
    }

    #[test]
    fn test_dealloc_page() {
        let config = DBConfig::load_db_config("config.json".to_string());
        let mut dm = DiskManager::new(&config);
        let page_id = PageId::new(0, 0);
        dm.dealloc_page(page_id);
        let expected_page_id = PageId::new(0, 0);
        assert!(dm.free_pages.contains(&expected_page_id));
    }

    #[test]
    fn test_save_state() {
        //POUR TESTER SAVE_STATE() IL FAUT RETIRER LE SAVE_STATE DE DEALLOC !!!
        let config = DBConfig::load_db_config("config.json".to_string());
        let mut dm = DiskManager::new(&config);

        let page_id = PageId::new(999, 0);
        dm.dealloc_page(page_id);
        let _ = dm.save_state();

        let dm2 = DiskManager::new(&config);
        let expected_page_id = PageId::new(999, 0);
        assert!(dm2.free_pages.contains(&expected_page_id));
    }

    #[test]
    fn test_load_state() {
        //TEST À REFAIRE

        let config = DBConfig::load_db_config("config.json".to_string());
        let _dm = DiskManager::new(&config);
    }
}
