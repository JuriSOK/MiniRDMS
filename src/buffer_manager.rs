use crate::buffer::Buffer;
use crate::{config::DBConfig, disk_manager::DiskManager, page::PageId, page_info::PageInfo};
use bytebuffer::ByteBuffer;
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;

pub struct BufferManager<'a> {
    //Quand on passe une référence e, attributs, life time obligatoire
    db_config: &'a DBConfig,
    disk_manager: RefCell<DiskManager<'a>>,

    //ça c'est pour stocker les infos sur les pages, notamment le moment où on les charges, le pin count etc
    liste_pages: Vec<PageInfo>, //quand c'est mutable aussi

    //Concrètement, c'est le buffer pool, Ex si 4 Buffers, alors on a un vecteur de 4 ByteBuffer, et les buffer c'est le contenu des pages, en fait c'est un peu comme notre ram ça
    liste_buffer: Vec<Rc<RefCell<ByteBuffer>>>,

    //Pour pouvoir tracker les pages à enlever, ex à chaque getPage on incrémente le temps, et si on doit freePage,
    //on sait que c'est à ce temps là. Ex : GetPage --> compteur_temps == 1 et pin_count  == 1, freePage --> compteur_temps = 0 et pin_count = 0 donc bye bye la page
    compteur_temps: u64,

    //pour le choix de l'algo de remplacement, peut-etre mettre une enum plus tard ?
    algo_remplacement: String,

    // Compteur général du nombre de page dans le buffer, utile pour charger les pages quand la liste de buffer n'est pas encore remplie
    nb_pages_vecteur: u32,
}

impl<'a> BufferManager<'a> {
    pub fn new(
        db_config: &'a DBConfig,
        disk_manager: DiskManager<'a>,
        algo_remplacement: String,
    ) -> Self {
        //dès qu'on créé le buffer_manager on initialise le compteur de temps
        let compteur_temps: u64 = 0;

        //On crée un Vecteur de ByteBuffer de la taille qu'on a dans le fichier.json
        let mut tmp: Vec<Rc<RefCell<ByteBuffer>>> =
            Vec::<Rc<RefCell<ByteBuffer>>>::with_capacity(db_config.get_bm_buffer_count() as usize);

        for _i in 0..db_config.get_bm_buffer_count() as usize {
            let buffer = Rc::new(RefCell::new(ByteBuffer::new()));
            buffer
                .borrow_mut()
                .resize(db_config.get_page_size() as usize);
            tmp.push(buffer);
        }

        //initialisation de la liste des pages, je me demande si on pouvait pas fusionner le buffer et ça, peut-être trop galère jsp
        let tmp2: Vec<PageInfo> =
            Vec::<PageInfo>::with_capacity(db_config.get_bm_buffer_count() as usize);

        /*
        for i in 0..db_config.get_bm_buffer_count() as usize{
            let page_info:Option<PageInfo> = None;
            tmp2.push(page_info.unwrap());
        }
        */

        Self {
            db_config,
            disk_manager: RefCell::new(disk_manager),
            liste_pages: tmp2,
            liste_buffer: tmp,
            compteur_temps,
            algo_remplacement,
            nb_pages_vecteur: 0, //0 pages dans le vecteur pour l'instant, se référer aux commentaires de l'attribut
        }
    }

    pub fn get_disk_manager_mut(&self) -> RefMut<DiskManager<'a>> {
        self.disk_manager.borrow_mut()
    }
    pub fn get_disk_manager(&self) -> RefMut<DiskManager<'a>> {
        self.disk_manager.borrow_mut()
    }

    pub fn get_db_config(&self) -> &DBConfig {
        return self.db_config;
    }

    pub fn get_liste_pages(&self) -> &Vec<PageInfo> {
        return &self.liste_pages;
    }

    pub fn get_liste_buffer(&self) -> &Vec<Rc<RefCell<ByteBuffer>>> {
        return &self.liste_buffer;
    }

    pub fn get_compteur_temps(&self) -> u64 {
        return self.compteur_temps; //pas besoin de référence ici je pense
    }

    pub fn get_algo(&self) -> String {
        return self.algo_remplacement.clone();
    }

    pub fn get_nb_pages_vecteur(&self) -> u32 {
        return self.nb_pages_vecteur;
    }

    pub fn lru(&mut self) -> usize {
        //Renvoie l'indice de la page à bouger dans liste_buffer (je pense mais à vérifier c pas qui a fait)
        //on va dire que c'est bon, grosse flemme de vérifier là tout de suite

        let mut indice: u32 = 0;

        //On définit arbitrairement la première page comme référence pour la comparaison.
        let mut oldest_page: &PageInfo = &self.liste_pages[0];

        let mut premierelemtrouve: bool = false;

        //On parcourt le buffer pool (vecteur de ByteBuffer)
        for i in 0..self.liste_pages.len() {
            //Si la page dans un ByteBuffer a le pincount à 0
            if self.liste_pages[i].get_pin_count() == 0 {
                //On cherche maintenant si il existe une page avec le pin_count() à 0 avec un temps plus petit,
                //Si c'est le cas alors on prend lui car on est dans LRU (Least recently use).
                if premierelemtrouve {
                    if oldest_page.get_time() > self.liste_pages[i].get_time() {
                        oldest_page = &self.liste_pages[i];
                        indice = i as u32;
                    }

                //On a trouvé une première page, donc on active premierelemtrouve pour commencé à comparer.
                } else {
                    oldest_page = &self.liste_pages[i];
                    premierelemtrouve = true;
                }
            }
        }

        //On doit vérifier car si on en trouve pas, cela va renvoyer la pageInfo qu'on a défini arbitrairement.
        if oldest_page.get_pin_count() == 0 {
            return indice as usize;
        } else {
            return self.db_config.get_bm_buffer_count() as usize; //ON a besoin d'une valeur de retour, ici valeur interdit à priori?
        }
    }

    //Même idée que LRU sauf qu'au lieu de prendre celle avec le temps le plus bas, on va prendre celui avec le temps le plus haut.
    pub fn mru(&mut self) -> usize {
        //Renvoie l'indice de la page à bouger dans liste_buffer (je pense mais à vérifier c pas qui a fait)

        let mut indice: u32 = 0;

        let mut oldest_page: &PageInfo = &self.liste_pages[0];

        let mut premierelemtrouve: bool = false;

        for i in 0..self.liste_pages.len() {
            if self.liste_pages[i].get_pin_count() == 0 {
                if premierelemtrouve {
                    if oldest_page.get_time() < self.liste_pages[i].get_time() {
                        oldest_page = &self.liste_pages[i];
                        indice = i as u32;
                    }
                } else {
                    oldest_page = &self.liste_pages[i];
                    premierelemtrouve = true;
                }
            }
        }

        if oldest_page.get_pin_count() == 0 {
            return indice as usize;
        } else {
            return self.db_config.get_bm_buffer_count() as usize; //ON a besoin d'une valeur de retour, ici valeur interdit à priori?
                                                                  //return 0;
        }
    }

    //Pour changer l'algo
    pub fn set_current_replacement_policy(&mut self, algo: String) {
        self.algo_remplacement = algo;
    }

    // ATTTENTION À VÉRIFIER ABSOLUMENT JE SUIS PAS CONFIANT DU TOUT POUR CA
    //visiblement cette version est mieux que l'ancienne, enfin elle est censé faire ce qu'il faut là, à voir si ça fonctionne
    pub fn get_page(&mut self, page_id: &PageId) -> Buffer {
        //le bloc if ici c'est dans le cas où le vecteur n'est pas encore rempli, il n'est pas nécessaire de faire tourner l'algo lru (encore ptet qu'on pouvait juste le faire tourner jsp) et on peut pas non plus parcourir la liste_pages pcq elle est vide
        if self.nb_pages_vecteur < self.db_config.get_bm_buffer_count() {
            for i in 0..self.liste_pages.len() {
                //on va regarder si on trouve pas la page voulue dans le buffer déjà, si c'est le cas pas besoin de la remettre dedans
                if page_id.get_file_idx() == self.liste_pages[i].get_page_id().get_file_idx()
                    && page_id.get_page_idx() == self.liste_pages[i].get_page_id().get_page_idx()
                {
                    // pin count ++ quand on est sûr que la page est bien allouée
                    let setpin = self.liste_pages[i].get_pin_count() + 1;
                    self.liste_pages[i].set_pin_count(setpin);
                    self.liste_pages[i].set_time(self.compteur_temps as i32); // à voir ça, il faut vérifier si on met le compteur au bon moment
                    self.compteur_temps += 1; //du coup on incrémente aussi le compteur de temps à la fin
                    return Buffer::new(&self.liste_buffer[i]);
                }
            }

            //là on créé un pageIngo du coup, avec les infos du pageID passé en paramètre, d'ailleurs on aurait pu juste rajouter des attributs dans le pageID et pas faire de pageInfo ? à méditer
            let pageinfo: PageInfo =
                PageInfo::new(page_id.clone(), 1, false, self.compteur_temps as i32); //ptet ça bloquera ici, à cause de compteur_temps, mais je suis confiant perso

            let ind: u32 = self.nb_pages_vecteur;
            //let mut list : ByteBuffer = self.liste_buffer[ind as usize];

            //là on met le page info au bon indice du coup, et on passe par une variable (constante plutôt) ind pcq on peut pas mettre deux self sur la même ligne

            /*
            self.liste_pages[ind as usize] = pageinfo;
            */
            self.liste_pages.push(pageinfo);
            //ça ça sert à mettre la page dans la liste des buffer du coup
            let _ = self
                .disk_manager
                .borrow()
                .read_page(&page_id, &mut self.liste_buffer[ind as usize].borrow_mut());
            //là on incrémente le nb_pages pour mettre la prochaine au bon endroit
            self.nb_pages_vecteur += 1;
            //on incrémente à chaque get_page du coup
            self.compteur_temps += 1; //A REVOIR ON LE SET JAMAIS DANS LA PAGE

            //on retourne le buffer correspondant
            return Buffer::new(&self.liste_buffer[ind as usize]);
        } else {
            //bloc else correspondant au cas ou la liste des buffer est remplie
            for i in 0..self.liste_pages.len() {
                //on va regarder si on trouve pas la page voulue dans le buffer déjà, si c'est le cas pas besoin de la remettre dedans
                if page_id.get_file_idx() == self.liste_pages[i].get_page_id().get_file_idx()
                    && page_id.get_page_idx() == self.liste_pages[i].get_page_id().get_page_idx()
                {
                    // pin count ++ quand on est sûr que la page est bien allouée
                    let setpin = self.liste_pages[i].get_pin_count() + 1;
                    self.liste_pages[i].set_pin_count(setpin);
                    self.liste_pages[i].set_time(self.compteur_temps as i32); // à voir ça, il faut vérifier si on met le compteur au bon moment
                    self.compteur_temps += 1; //du coup on incrémente aussi le compteur de temps à la fin
                    return Buffer::new(&self.liste_buffer[i]);
                }
            }

            //Pour le cas où un une page est à remplacer --> indice de la page à changer
            let page_a_changer: usize;

            //les algos retournent juste l'indice de la page à remplacer, pas la page en elle-même
            if self.algo_remplacement.eq("LRU") {
                page_a_changer = self.lru();
            } else {
                page_a_changer = self.mru();
            }
            if self.liste_pages[page_a_changer].get_pin_count() == 0 {
                if self.liste_pages[page_a_changer].get_dirty() == true {
                    let _ = self.disk_manager.borrow().write_page(
                        &self.liste_pages[page_a_changer].get_page_id(),
                        &mut self.liste_buffer[page_a_changer].borrow_mut(),
                    );
                }

                self.liste_buffer[page_a_changer].borrow_mut().clear();
                let _ = self.disk_manager.borrow().write_page(
                    &page_id,
                    &mut self.liste_buffer[page_a_changer].borrow_mut(),
                );

                let _ = self.disk_manager.borrow().read_page(
                    &page_id,
                    &mut self.liste_buffer[page_a_changer].borrow_mut(),
                );
                let pageinfo: PageInfo =
                    PageInfo::new(page_id.clone(), 1, false, self.compteur_temps as i32);
                self.liste_pages[page_a_changer] = pageinfo; //il faut mettre le page info correspondant dans la liste des pages
            }
            self.compteur_temps += 1; //on incrémente le compteur de temps du coup
            return Buffer::new(&self.liste_buffer[page_a_changer]);
        }
    }

    // ATTTENTION À VÉRIFIER ABSOLUMENT JE SUIS PAS CONFIANT DU TOUT POUR CA

    pub fn free_page(&mut self, page_id: &PageId, bit_dirty: bool) -> () {
        //self.compteur_temps+=1; on incrémente pas le temps quand on fait un free
        let mut page_info: &mut PageInfo = &mut PageInfo::new(page_id.clone(), 0, false, 0); //CETTE LIGNE GROS GROS PROBLEME, AU NIVEAU LIFE TIME C'EST UNE DINGUERIE
        let mut trouve: bool = false;
        for i in self.liste_pages.iter_mut() {
            if page_id.get_file_idx() == i.get_page_id().get_file_idx()
                && page_id.get_page_idx() == i.get_page_id().get_page_idx()
            {
                page_info = i;
                trouve = true;
                break;
            }
        }
        if !trouve {
            return;
        }
        let index = page_info.get_pin_count() - 1;
        page_info.set_pin_count(index);
        page_info.set_dirty_bit(bit_dirty);
        if page_info.get_pin_count() == 0 {
            page_info.set_time(self.compteur_temps as i32);
        }
    }

    pub fn flush_buffers(&mut self) {
        for i in 0..self.nb_pages_vecteur {
            if self.liste_pages[i as usize].get_dirty() == true {
                let _ = self.disk_manager.borrow().write_page(
                    self.liste_pages[i as usize].get_page_id(),
                    &mut self.liste_buffer[i as usize].borrow_mut(),
                );
            }
            self.liste_pages[i as usize].set_pin_count(0);
            self.liste_pages[i as usize].set_dirty_bit(false);
            self.liste_pages[i as usize].set_time(0);
        }
        self.nb_pages_vecteur = 0;
        self.liste_pages.clear();
        // self.liste_buffer.clear();

        let mut tmp: Vec<Rc<RefCell<ByteBuffer>>> = Vec::<Rc<RefCell<ByteBuffer>>>::with_capacity(
            self.db_config.get_bm_buffer_count() as usize,
        );

        for _i in 0..self.db_config.get_bm_buffer_count() as usize {
            let buffer = Rc::new(RefCell::new(ByteBuffer::new()));
            buffer
                .borrow_mut()
                .resize(self.db_config.get_page_size() as usize);
            tmp.push(buffer);
        }

        self.liste_buffer = tmp;
    }

    pub fn afficher_etat_buffer(&self) {
        println!(
            "Mon buffer possède actuellement en ram {} pages",
            self.liste_pages.len()
        );

        for i in 0..self.liste_pages.len() {
            println!(
                "Emplacement {} : File idx : {}, Page idx: {}",
                i,
                self.liste_pages[i].get_page_id().get_file_idx(),
                self.liste_pages[i].get_page_id().get_page_idx()
            );
            println!("Son pin-count : {}", self.liste_pages[i].get_pin_count());
            println!("Son bit dirty : {}", self.liste_pages[i].get_dirty());
            println!("Le contenu du buffer : {:?}", self.liste_buffer[i]);
        }

        if self.liste_pages.len() == 0 {
            println!("Il y'a rien dans notre ram");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;
    //premier test, on commence gentiment juste pour voir si le constructeur fonctionne bien
    #[test]
    fn test_constructeur_buffer() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let dm = DiskManager::new(&config);

        let algo_lru = String::from("LRU");

        let buffer_manager = BufferManager::new(&config, dm, algo_lru);
        assert_eq!(
            buffer_manager.get_liste_buffer().len(),
            config.get_bm_buffer_count() as usize
        );
        assert_eq!(buffer_manager.get_nb_pages_vecteur(), 0);
        assert_eq!(buffer_manager.get_algo(), "LRU");
    }

    //a l'heure où j'écris ces lignes je suis en sueur
    //le but du test c'est de voir si déjà on arrive à mettre les pages dans le buffer et si ensuite on trouve les bonnes; enfin ça on verra après

    #[test]
    fn test_flush_buffer() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let mut dm = DiskManager::new(&config);
        let algo_lru = String::from("LRU");

        let pagea = dm.alloc_page();
        let pageb = dm.alloc_page();
        let pagec = dm.alloc_page();
        let paged = dm.alloc_page();
        let pagee = dm.alloc_page();

        let mut buffer_manager = BufferManager::new(&config, dm, algo_lru); //SI ON MET LES EMRPUNTS MUTABLES AVANT LES EMPRUNTS IMMUTABLES CA FONCTIONNE MAIS IL FAUT ABSOLUMENT TROUVER UNE AUTRE SOLUTION SINON ON EST CUIT

        //comme on a pas vraiment de manière d'enregistrer les infos pour l'instant on fait ça à la main
        //du coup ça logiquement c'est pour la page a et b

        let mut buffer1 = Vec::new();
        let _ = buffer1.write_all("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".as_bytes());
        let _ = buffer1.write_all("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".as_bytes());

        let num1 = pagea.get_file_idx();
        let nomfichier1 = format!("res/dbpath/BinData/F{num1}.rsdb");
        println!("{}", nomfichier1);
        let mut fichier1 = OpenOptions::new()
            .write(true)
            .open(nomfichier1)
            .expect("tkt");
        let _ = fichier1.write_all(&buffer1);

        //là c'est pour la page c et d

        let mut buffer2 = Vec::new();
        let _ = buffer2.write_all("cccccccccccccccccccccccccccccccc".as_bytes());
        let _ = buffer2.write_all("dddddddddddddddddddddddddddddddd".as_bytes());
        let num2 = pagec.get_file_idx();
        let nomfichier2 = format!("res/dbpath/BinData/F{num2}.rsdb");
        println!("{}", nomfichier2);
        let mut fichier2 = OpenOptions::new()
            .write(true)
            .open(nomfichier2)
            .expect("tkt");
        let _ = fichier2.write_all(&buffer2);

        //là pour la page e

        let mut buffer3 = Vec::new();
        let _ = buffer3.write_all("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".as_bytes());
        let num3 = pagee.get_file_idx();
        let nomfichier3 = format!("res/dbpath/BinData/F{num3}.rsdb");
        println!("{}", nomfichier3);
        let mut fichier3 = OpenOptions::new()
            .write(true)
            .open(nomfichier3)
            .expect("tkt");
        let _ = fichier3.write_all(&buffer3);

        //d'après moi on devrait avoir 3 fichiers mais visiblement on en a qu'un seul et aucune erreur, ptet que j'ai fait n'importe quoi mais faudra regarder la taille des fichiers au cas où

        let bytebuffer_de_pagea = buffer_manager.get_page(&pagea);
        let bytebuffer_de_pageb = buffer_manager.get_page(&pageb);
        let bytebuffer_de_pagec = buffer_manager.get_page(&pagec);
        let bytebuffer_de_paged = buffer_manager.get_page(&paged);
        buffer_manager.free_page(&pagea, false);
        let bytebuffer_de_pagee = buffer_manager.get_page(&pagee);

        buffer_manager.flush_buffers();
        assert_eq!(buffer_manager.get_nb_pages_vecteur(), 0);
    }

    #[test]
    //cargo test test_get_page -- --show-output
    fn test_get_page_and_free_page() {
        let s: String = String::from("config.json");
        let config = DBConfig::load_db_config(s);
        let mut dm = DiskManager::new(&config);
        let algo_lru = String::from("LRU");

        let pagea = dm.alloc_page();
        let pageb = dm.alloc_page();
        let pagec = dm.alloc_page();
        let paged = dm.alloc_page();
        let pagee = dm.alloc_page();

        let mut buffer_manager = BufferManager::new(&config, dm, algo_lru); //SI ON MET LES EMRPUNTS MUTABLES AVANT LES EMPRUNTS IMMUTABLES CA FONCTIONNE MAIS IL FAUT ABSOLUMENT TROUVER UNE AUTRE SOLUTION SINON ON EST CUIT

        //comme on a pas vraiment de manière d'enregistrer les infos pour l'instant on fait ça à la main
        //du coup ça logiquement c'est pour la page a et b

        let mut buffer1 = ByteBuffer::new();
        let _ = buffer1.write_all("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".as_bytes());
        let _ = buffer1.write_all("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".as_bytes());
        let data1 = buffer1.as_bytes();
        let num1 = pagea.get_file_idx();
        let nomfichier1 = format!("res/dbpath/BinData/F{num1}.rsdb");
        println!("{}", nomfichier1);
        let mut fichier1 = OpenOptions::new()
            .append(true)
            .write(true)
            .open(nomfichier1)
            .expect("tkt");
        let _ = fichier1.write_all(&data1);

        //là c'est pour la page c et d

        let mut buffer2 = ByteBuffer::new();
        let _ = buffer2.write_all("cccccccccccccccccccccccccccccccc".as_bytes());
        let _ = buffer2.write_all("dddddddddddddddddddddddddddddddd".as_bytes());

        let data2 = buffer2.as_bytes();
        let num2 = pagec.get_file_idx();
        let nomfichier2 = format!("res/dbpath/BinData/F{num2}.rsdb");
        println!("{}", nomfichier2);
        let mut fichier2 = OpenOptions::new()
            .append(true)
            .write(true)
            .open(nomfichier2)
            .expect("tkt");
        let _ = fichier2.write_all(&data2);

        //là pour la page e

        let mut buffer3 = ByteBuffer::new();
        let _ = buffer3.write_all("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".as_bytes());

        let data3 = buffer3.as_bytes();
        let num3 = pagee.get_file_idx();
        let nomfichier3 = format!("res/dbpath/BinData/F{num3}.rsdb");
        println!("{}", nomfichier3);
        let mut fichier3 = OpenOptions::new()
            .write(true)
            .open(nomfichier3)
            .expect("tkt");
        let _ = fichier3.write_all(&data3);

        //d'après moi on devrait avoir 3 fichiers mais visiblement on en a qu'un seul et aucune erreur, ptet que j'ai fait n'importe quoi mais faudra regarder la taille des fichiers au cas où

        let bytebuffer_de_pagea = buffer_manager.get_page(&pagea);
        let bytebuffer_de_pageb = buffer_manager.get_page(&pageb);
        let bytebuffer_de_pagec = buffer_manager.get_page(&pagec);
        let bytebuffer_de_paged = buffer_manager.get_page(&paged);
        buffer_manager.free_page(&pagea, false);
        let mut bytebuffer_de_pagee = buffer_manager.get_page(&pagee);

        let buffer3 = buffer_manager.liste_buffer[3].borrow();
        let buffer1 = buffer_manager.liste_buffer[1].borrow();
        let buffer2 = buffer_manager.liste_buffer[2].borrow();

        //let bytebuffer_test = bytebuffer_de_pagea.clone(); ICI JE TEST LE CONTENUE D'UN BYTEBUFFER POUR VOIR SI
        //GET PAGE RENVOIE BIEN UN VECTEUR<U8> <=> BYTEBUFFER

        let mut fichier_test = OpenOptions::new()
            .write(true) // Ouvre en mode écriture
            .create(true) // Crée le fichier s'il n'existe pas
            .truncate(true) // Tronque le fichier s'il existe (écrase le contenu)
            .open("res/fichier_test_buffermanager")
            .expect("Erreur lors de l'ouverture du fichier");

        fichier_test
            .write_all(&buffer3.as_bytes())
            .expect("Erreur lors de l'écriture des données");
        fichier_test
            .write_all(&buffer1.as_bytes())
            .expect("Erreur lors de l'écriture des données");
        fichier_test
            .write_all(&buffer2.as_bytes())
            .expect("Erreur lors de l'écriture des données");

        //fichier_test.write_all(&bytebuffer_test); JE TESTE LE CONTENU D'UN BYTEBUFFER

        //println!("{}", buffer_manager.liste_buffer[3].read_string().unwrap());
        //println!("{}", buffer_manager.liste_buffer[1].read_string().unwrap());
        //jprintln!("{}", buffer_manager.liste_buffer[2].read_string().unwrap());
        //println!("{}", buffer_manager.liste_buffer[0].read_string().unwrap());
    }
}
