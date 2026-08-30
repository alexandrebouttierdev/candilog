import type {
  Certification,
  Education,
  Experience,
  Identity,
  Language,
  Project,
  Skill,
} from "@/shared/types/generated/profile";

const text = (value: string | null) => value ?? "";

export const identityDefaults = (value: Identity) => ({
  ...value,
  phone: text(value.phone),
  city: text(value.city),
  title: text(value.title),
  resume: text(value.resume),
  linkedin: text(value.linkedin),
  github: text(value.github),
  website: text(value.website),
});

export const emptyExperience = () => ({
  title: "",
  company: "",
  location: "",
  start_date: "",
  end_date: "",
  current: false,
  description: "",
});
export const experienceDefaults = (items: Experience[]) => ({
  items: items.map((item) => ({
    ...item,
    location: text(item.location),
    end_date: text(item.end_date),
    description: text(item.description),
  })),
});

export const skillDefaults = (items: Skill[]) => ({ items: structuredClone(items) });
export const emptySkill = () => ({ name: "" });

export const emptyEducation = () => ({
  degree: "",
  school: "",
  location: "",
  start_date: "",
  end_date: "",
  description: "",
});
export const educationDefaults = (items: Education[]) => ({
  items: items.map((item) => ({
    ...item,
    location: text(item.location),
    start_date: text(item.start_date),
    end_date: text(item.end_date),
    description: text(item.description),
  })),
});

export const languageDefaults = (items: Language[]) => ({ items: structuredClone(items) });
export const emptyLanguage = () => ({ name: "", level: "" });

export const emptyProject = () => ({ name: "", description: "", url: "", technologies: "" });
export const projectDefaults = (items: Project[]) => ({
  items: items.map((item) => ({
    ...item,
    description: text(item.description),
    url: text(item.url),
    technologies: text(item.technologies),
  })),
});

export const emptyCertification = () => ({ name: "", issuer: "", date: "", url: "" });
export const certificationDefaults = (items: Certification[]) => ({
  items: items.map((item) => ({
    ...item,
    issuer: text(item.issuer),
    date: text(item.date),
    url: text(item.url),
  })),
});
